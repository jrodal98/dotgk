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

use crate::gatekeeper::evaluate_gatekeeper_by_name;
use crate::gatekeeper::find_all_gatekeepers;
use crate::gatekeeper::get_config_dir;
use crate::gatekeeper::load_gatekeeper;

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
                if !is_cache_entry_expired(entry, current_timestamp) {
                    info!("Found valid cache entry for '{}': {}", name, entry.value);
                    println!("{}", entry.value);
                    return Ok(());
                } else {
                    info!("Cache entry for '{}' has expired, re-evaluating", name);
                }
            } else {
                info!("No cache entry found for '{}', evaluating", name);
            }
        }
    } else {
        info!("No cache file found, evaluating '{}'", name);
    }

    // Cache miss or expired - evaluate and cache
    let result = evaluate_gatekeeper_by_name(&name)?;
    info!("Evaluation result: {}", result);
    println!("{}", result);

    // Load gatekeeper to get TTL configuration and cache the result
    let gatekeeper = load_gatekeeper(&name)?;
    let ttl = gatekeeper.ttl;

    if let Err(e) = cache_result_with_ttl(&name, result, cache_path, UpdateType::Evaluate, ttl) {
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
            });

        if should_evaluate {
            info!("Evaluating gatekeeper: {}", name);
            match evaluate_gatekeeper_by_name(&name) {
                Ok(result) => {
                    // Load gatekeeper to get TTL configuration
                    let gatekeeper = load_gatekeeper(&name)?;
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
