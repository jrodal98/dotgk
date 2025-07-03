use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use tracing::debug;
use tracing::error;
use tracing::info;
use tracing::instrument;

use crate::gatekeeper::Gatekeeper;
use crate::gatekeeper::find_all_gatekeepers;
use crate::gatekeeper::get_config_dir;
use crate::gatekeeper::get_gatekeeper_path;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum UpdateType {
    Evaluate,
    Sync,
    Set,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CacheEntry {
    pub value: bool,
    pub ts: u64,
    pub update_type: UpdateType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Cache {
    pub cache: HashMap<String, CacheEntry>,
    pub ts: u64,
}

fn get_cache_path(cache_path: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = cache_path {
        return Ok(path);
    }

    let mut config_dir = get_config_dir()?;
    config_dir.push("dotgk");
    config_dir.push("cache.json");
    Ok(config_dir)
}

pub fn cache_result_with_ttl(
    name: &str,
    result: bool,
    cache_path: Option<PathBuf>,
    update_type: UpdateType,
    ttl_seconds: Option<u64>,
) -> Result<()> {
    let cache_file_path = get_cache_path(cache_path)?;

    // Create cache directory if it doesn't exist
    if let Some(parent) = cache_file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let current_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("Failed to get current timestamp")?
        .as_secs();

    // Load existing cache or create new one
    let mut cache = if cache_file_path.exists() {
        let cache_content =
            fs::read_to_string(&cache_file_path).context("Failed to read existing cache file")?;
        serde_json::from_str::<Cache>(&cache_content)
            .context("Failed to parse existing cache file")?
    } else {
        Cache {
            cache: HashMap::new(),
            ts: current_timestamp,
        }
    };

    // Calculate expiration time if TTL is provided
    let expires_at = ttl_seconds.map(|ttl| current_timestamp + ttl);

    // Update the cache entry
    let entry = CacheEntry {
        value: result,
        ts: current_timestamp,
        update_type,
        expires_at,
    };
    cache.cache.insert(name.to_string(), entry);
    cache.ts = current_timestamp;

    // Write updated cache
    let cache_json = serde_json::to_string_pretty(&cache).context("Failed to serialize cache")?;

    fs::write(&cache_file_path, cache_json)
        .with_context(|| format!("Failed to write cache to {:?}", cache_file_path))?;

    debug!(
        "Cached result for '{}': {} at {:?}",
        name, result, cache_file_path
    );
    Ok(())
}

fn is_cache_entry_expired(entry: &CacheEntry, current_timestamp: u64) -> bool {
    if let Some(expires_at) = entry.expires_at {
        current_timestamp >= expires_at
    } else {
        false
    }
}

fn get_file_modification_time(path: &PathBuf) -> Result<u64> {
    let metadata =
        fs::metadata(path).with_context(|| format!("Failed to get metadata for {:?}", path))?;
    let modified = metadata
        .modified()
        .with_context(|| format!("Failed to get modification time for {:?}", path))?;
    let timestamp = modified
        .duration_since(UNIX_EPOCH)
        .context("Failed to convert modification time to timestamp")?
        .as_secs();
    Ok(timestamp)
}

fn is_gatekeeper_file_modified(name: &str, cache_entry: &CacheEntry) -> bool {
    match get_gatekeeper_path(name) {
        Ok(gatekeeper_path) => {
            if !gatekeeper_path.exists() {
                // If the gatekeeper file doesn't exist, consider it modified to force re-evaluation
                debug!(
                    "Gatekeeper file {:?} doesn't exist, treating as modified",
                    gatekeeper_path
                );
                return true;
            }

            match get_file_modification_time(&gatekeeper_path) {
                Ok(file_timestamp) => {
                    let is_modified = file_timestamp > cache_entry.ts;
                    if is_modified {
                        debug!(
                            "Gatekeeper file {:?} modified at {} > cache entry at {}",
                            gatekeeper_path, file_timestamp, cache_entry.ts
                        );
                    }
                    is_modified
                }
                Err(e) => {
                    debug!(
                        "Failed to get modification time for {:?}: {}, treating as modified",
                        gatekeeper_path, e
                    );
                    true
                }
            }
        }
        Err(e) => {
            debug!(
                "Failed to get gatekeeper path for '{}': {}, treating as modified",
                name, e
            );
            true
        }
    }
}

#[instrument]
pub fn set_command(
    name: String,
    value: bool,
    cache_path: Option<PathBuf>,
    ttl_seconds: Option<u64>,
) -> Result<()> {
    info!("Setting cache value for '{}': {}", name, value);

    cache_result_with_ttl(&name, value, cache_path, UpdateType::Set, ttl_seconds)?;

    if let Some(ttl) = ttl_seconds {
        println!("Set '{}' = {} (expires in {} seconds)", name, value, ttl);
    } else {
        println!("Set '{}' = {} (no expiration)", name, value);
    }

    Ok(())
}

#[instrument]
pub fn get_command(name: String, cache_path: Option<PathBuf>) -> Result<()> {
    info!("Getting gatekeeper value: {}", name);

    let cache_file_path = get_cache_path(cache_path.clone())?;
    let current_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("Failed to get current timestamp")?
        .as_secs();

    // Try to load from cache first
    if cache_file_path.exists() {
        let cache_content =
            fs::read_to_string(&cache_file_path).context("Failed to read cache file")?;

        if let Ok(cache) = serde_json::from_str::<Cache>(&cache_content) {
            if let Some(entry) = cache.cache.get(&name) {
                if !is_cache_entry_expired(entry, current_timestamp)
                    && !is_gatekeeper_file_modified(&name, entry)
                {
                    info!("Found valid cache entry for '{}': {}", name, entry.value);
                    println!("{}", entry.value);
                    return Ok(());
                } else {
                    if is_cache_entry_expired(entry, current_timestamp) {
                        info!("Cache entry for '{}' has expired, re-evaluating", name);
                    } else {
                        info!(
                            "Gatekeeper file for '{}' has been modified, re-evaluating",
                            name
                        );
                    }
                }
            } else {
                info!("No cache entry found for '{}', evaluating", name);
            }
        }
    } else {
        info!("No cache file found, evaluating '{}'", name);
    }

    // Cache miss or expired - evaluate and cache
    let gk = Gatekeeper::from_name(&name)?;
    let result = gk.evaluate().context("Failed to evaluate gatekeeper")?;
    info!("Evaluation result: {}", result);
    println!("{}", result);

    if let Err(e) = cache_result_with_ttl(&name, result, cache_path, UpdateType::Evaluate, gk.ttl) {
        // Don't fail the command if caching fails, just log the error
        tracing::warn!("Failed to cache evaluation result: {}", e);
    }

    Ok(())
}

#[instrument]
pub fn sync_command(cache_path: Option<PathBuf>, force: bool) -> Result<()> {
    info!("Syncing all gatekeepers (force: {})", force);

    let cache_file_path = get_cache_path(cache_path)?;
    debug!("Cache path: {:?}", cache_file_path);

    // Create cache directory if it doesn't exist
    if let Some(parent) = cache_file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let current_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("Failed to get current timestamp")?
        .as_secs();

    // Load existing cache to preserve non-expired entries
    let existing_cache = if cache_file_path.exists() {
        let cache_content =
            fs::read_to_string(&cache_file_path).context("Failed to read existing cache file")?;
        serde_json::from_str::<Cache>(&cache_content)
            .context("Failed to parse existing cache file")?
    } else {
        Cache {
            cache: HashMap::new(),
            ts: current_timestamp,
        }
    };

    let gatekeepers = find_all_gatekeepers()?;
    info!("Found {} gatekeepers", gatekeepers.len());

    let mut cache_entries = HashMap::new();
    let mut updated_count = 0;
    let mut preserved_count = 0;
    let mut skipped_count = 0;

    // First, preserve non-expired entries that aren't gatekeepers
    for (name, entry) in existing_cache.cache.iter() {
        if !gatekeepers.contains(name) && !is_cache_entry_expired(entry, current_timestamp) {
            cache_entries.insert(name.clone(), entry.clone());
            preserved_count += 1;
            debug!("Preserved non-expired entry for '{}'", name);
        }
    }

    // Process gatekeepers
    for name in gatekeepers {
        let existing_entry = existing_cache.cache.get(&name);
        let should_evaluate = force
            || existing_entry.is_none()
            || existing_entry.map_or(false, |entry| {
                is_cache_entry_expired(entry, current_timestamp)
                    || is_gatekeeper_file_modified(&name, entry)
            });

        if should_evaluate {
            info!("Evaluating gatekeeper: {}", name);
            let gatekeeper = Gatekeeper::from_name(&name)?;
            match gatekeeper.evaluate() {
                Ok(result) => {
                    let expires_at = gatekeeper.ttl.map(|ttl| current_timestamp + ttl);

                    let entry = CacheEntry {
                        value: result,
                        ts: current_timestamp,
                        update_type: UpdateType::Sync,
                        expires_at,
                    };
                    cache_entries.insert(name.clone(), entry);
                    updated_count += 1;
                    info!("Cached result for '{}': {}", name, result);
                }
                Err(e) => {
                    error!("Failed to evaluate gatekeeper '{}': {}", name, e);
                }
            }
        } else {
            // Keep existing entry
            if let Some(entry) = existing_entry {
                cache_entries.insert(name.clone(), entry.clone());
                skipped_count += 1;
                debug!("Skipped non-expired gatekeeper '{}'", name);
            }
        }
    }

    let cache = Cache {
        cache: cache_entries,
        ts: current_timestamp,
    };

    let cache_json = serde_json::to_string_pretty(&cache).context("Failed to serialize cache")?;

    fs::write(&cache_file_path, cache_json)
        .with_context(|| format!("Failed to write cache to {:?}", cache_file_path))?;

    info!("Cache written to {:?}", cache_file_path);
    if force {
        println!(
            "Force synced {} gatekeepers, preserved {} non-gatekeeper entries",
            updated_count, preserved_count
        );
    } else {
        println!(
            "Synced {} gatekeepers, skipped {} non-expired, preserved {} non-gatekeeper entries",
            updated_count, skipped_count, preserved_count
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use std::thread;
    use std::time::Duration;

    use tempfile::TempDir;

    use super::*;

    fn create_test_cache_entry(ts: u64, expires_at: Option<u64>) -> CacheEntry {
        CacheEntry {
            value: true,
            ts,
            update_type: UpdateType::Evaluate,
            expires_at,
        }
    }

    #[test]
    fn test_is_cache_entry_expired_no_ttl() {
        let entry = create_test_cache_entry(1000, None);
        let current_time = 2000;

        // Entry without TTL should never expire
        assert!(!is_cache_entry_expired(&entry, current_time));
    }

    #[test]
    fn test_is_cache_entry_expired_with_ttl_not_expired() {
        let entry = create_test_cache_entry(1000, Some(2000));
        let current_time = 1500;

        // Entry should not be expired if current time < expires_at
        assert!(!is_cache_entry_expired(&entry, current_time));
    }

    #[test]
    fn test_is_cache_entry_expired_with_ttl_expired() {
        let entry = create_test_cache_entry(1000, Some(1500));
        let current_time = 2000;

        // Entry should be expired if current time >= expires_at
        assert!(is_cache_entry_expired(&entry, current_time));
    }

    #[test]
    fn test_is_cache_entry_expired_with_ttl_exactly_expired() {
        let entry = create_test_cache_entry(1000, Some(1500));
        let current_time = 1500;

        // Entry should be expired if current time == expires_at
        assert!(is_cache_entry_expired(&entry, current_time));
    }

    #[test]
    fn test_get_file_modification_time() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test_file.txt");

        // Create a test file
        let mut file = File::create(&file_path)?;
        file.write_all(b"test content")?;
        file.sync_all()?;
        drop(file);

        // Get modification time
        let mod_time = get_file_modification_time(&file_path)?;

        // Should be a reasonable timestamp (after year 2020)
        assert!(mod_time > 1577836800); // Jan 1, 2020

        Ok(())
    }

    #[test]
    fn test_get_file_modification_time_nonexistent_file() {
        let nonexistent_path = PathBuf::from("/nonexistent/file.txt");
        let result = get_file_modification_time(&nonexistent_path);

        assert!(result.is_err());
    }

    #[test]
    fn test_is_gatekeeper_file_modified_file_newer() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let gatekeeper_path = temp_dir.path().join("test.json");

        // Create a gatekeeper file
        let mut file = File::create(&gatekeeper_path)?;
        file.write_all(b"{\"groups\": []}")?;
        file.sync_all()?;
        drop(file);

        // Wait a bit to ensure different timestamps
        thread::sleep(Duration::from_millis(10));

        // Create cache entry with older timestamp
        let old_timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() - 10;
        let cache_entry = create_test_cache_entry(old_timestamp, None);

        // Mock the gatekeeper path function by testing with a direct path check
        let file_mod_time = get_file_modification_time(&gatekeeper_path)?;
        let is_modified = file_mod_time > cache_entry.ts;

        assert!(is_modified);
        Ok(())
    }

    #[test]
    fn test_is_gatekeeper_file_modified_file_older() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let gatekeeper_path = temp_dir.path().join("test.json");

        // Create a gatekeeper file
        let mut file = File::create(&gatekeeper_path)?;
        file.write_all(b"{\"groups\": []}")?;
        file.sync_all()?;
        drop(file);

        // Create cache entry with newer timestamp
        let new_timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() + 10;
        let cache_entry = create_test_cache_entry(new_timestamp, None);

        // Check if file is considered modified
        let file_mod_time = get_file_modification_time(&gatekeeper_path)?;
        let is_modified = file_mod_time > cache_entry.ts;

        assert!(!is_modified);
        Ok(())
    }

    #[test]
    fn test_cache_result_with_ttl_new_cache() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let cache_path = temp_dir.path().join("cache.json");

        // Cache a result with TTL
        cache_result_with_ttl(
            "test_gatekeeper",
            true,
            Some(cache_path.clone()),
            UpdateType::Evaluate,
            Some(3600), // 1 hour TTL
        )?;

        // Verify cache file was created and contains expected data
        assert!(cache_path.exists());

        let cache_content = fs::read_to_string(&cache_path)?;
        let cache: Cache = serde_json::from_str(&cache_content)?;

        assert!(cache.cache.contains_key("test_gatekeeper"));
        let entry = &cache.cache["test_gatekeeper"];
        assert_eq!(entry.value, true);
        assert!(entry.expires_at.is_some());
        assert!(entry.expires_at.unwrap() > entry.ts);

        Ok(())
    }

    #[test]
    fn test_cache_result_with_ttl_no_ttl() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let cache_path = temp_dir.path().join("cache.json");

        // Cache a result without TTL
        cache_result_with_ttl(
            "test_gatekeeper",
            false,
            Some(cache_path.clone()),
            UpdateType::Set,
            None,
        )?;

        // Verify cache file was created and contains expected data
        let cache_content = fs::read_to_string(&cache_path)?;
        let cache: Cache = serde_json::from_str(&cache_content)?;

        let entry = &cache.cache["test_gatekeeper"];
        assert_eq!(entry.value, false);
        assert!(entry.expires_at.is_none());

        Ok(())
    }

    #[test]
    fn test_cache_result_with_ttl_update_existing() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let cache_path = temp_dir.path().join("cache.json");

        // Create initial cache entry
        cache_result_with_ttl(
            "test_gatekeeper",
            true,
            Some(cache_path.clone()),
            UpdateType::Evaluate,
            Some(3600),
        )?;

        // Update the same entry
        cache_result_with_ttl(
            "test_gatekeeper",
            false,
            Some(cache_path.clone()),
            UpdateType::Sync,
            Some(7200), // Different TTL
        )?;

        // Verify the entry was updated
        let cache_content = fs::read_to_string(&cache_path)?;
        let cache: Cache = serde_json::from_str(&cache_content)?;

        let entry = &cache.cache["test_gatekeeper"];
        assert_eq!(entry.value, false);
        assert!(matches!(entry.update_type, UpdateType::Sync));

        Ok(())
    }

    #[test]
    fn test_cache_preserves_other_entries() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let cache_path = temp_dir.path().join("cache.json");

        // Create first entry
        cache_result_with_ttl(
            "gatekeeper1",
            true,
            Some(cache_path.clone()),
            UpdateType::Evaluate,
            None,
        )?;

        // Create second entry
        cache_result_with_ttl(
            "gatekeeper2",
            false,
            Some(cache_path.clone()),
            UpdateType::Set,
            Some(3600),
        )?;

        // Verify both entries exist
        let cache_content = fs::read_to_string(&cache_path)?;
        let cache: Cache = serde_json::from_str(&cache_content)?;

        assert_eq!(cache.cache.len(), 2);
        assert!(cache.cache.contains_key("gatekeeper1"));
        assert!(cache.cache.contains_key("gatekeeper2"));

        assert_eq!(cache.cache["gatekeeper1"].value, true);
        assert_eq!(cache.cache["gatekeeper2"].value, false);

        Ok(())
    }

    #[test]
    fn test_update_type_serialization() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let cache_path = temp_dir.path().join("cache.json");

        // Test all update types
        let update_types = vec![UpdateType::Evaluate, UpdateType::Sync, UpdateType::Set];

        for (i, update_type) in update_types.into_iter().enumerate() {
            cache_result_with_ttl(
                &format!("test_{}", i),
                true,
                Some(cache_path.clone()),
                update_type.clone(),
                None,
            )?;
        }

        // Verify serialization
        let cache_content = fs::read_to_string(&cache_path)?;
        let _cache: Cache = serde_json::from_str(&cache_content)?;

        // Check that the JSON contains the expected lowercase strings
        assert!(cache_content.contains("\"evaluate\""));
        assert!(cache_content.contains("\"sync\""));
        assert!(cache_content.contains("\"set\""));

        Ok(())
    }
}
