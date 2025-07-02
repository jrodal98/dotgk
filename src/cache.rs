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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum UpdateType {
    Evaluate,
    Sync,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CacheEntry {
    pub value: bool,
    pub ts: u64,
    pub update_type: UpdateType,
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

#[instrument]
pub fn sync_command(cache_path: Option<PathBuf>) -> Result<()> {
    info!("Syncing all gatekeepers");

    let cache_file_path = get_cache_path(cache_path)?;
    debug!("Cache path: {:?}", cache_file_path);

    // Create cache directory if it doesn't exist
    if let Some(parent) = cache_file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let gatekeepers = find_all_gatekeepers()?;
    info!("Found {} gatekeepers", gatekeepers.len());

    let current_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("Failed to get current timestamp")?
        .as_secs();

    let mut cache_entries = HashMap::new();

    for name in gatekeepers {
        info!("Evaluating gatekeeper: {}", name);
        match evaluate_gatekeeper_by_name(&name) {
            Ok(result) => {
                let entry = CacheEntry {
                    value: result,
                    ts: current_timestamp,
                    update_type: UpdateType::Sync,
                };
                cache_entries.insert(name.clone(), entry);
                info!("Cached result for '{}': {}", name, result);
            }
            Err(e) => {
                error!("Failed to evaluate gatekeeper '{}': {}", name, e);
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
    println!("Synced {} gatekeepers to cache", cache.cache.len());
    Ok(())
}
