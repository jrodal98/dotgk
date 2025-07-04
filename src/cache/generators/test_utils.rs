#[cfg(test)]
use std::collections::HashMap;

#[cfg(test)]
use crate::cache::cache::Cache;
#[cfg(test)]
use crate::cache::cache::CacheEntry;
#[cfg(test)]
use crate::cache::cache::UpdateType;

#[cfg(test)]
pub fn create_test_cache() -> Cache {
    let mut cache_entries = HashMap::new();
    cache_entries.insert(
        "test-gk".to_string(),
        CacheEntry {
            value: true,
            ts: 1000,
            update_type: UpdateType::Sync,
            expires_at: None,
        },
    );
    cache_entries.insert(
        "another_gk".to_string(),
        CacheEntry {
            value: false,
            ts: 1000,
            update_type: UpdateType::Sync,
            expires_at: None,
        },
    );

    Cache {
        cache: cache_entries,
        ts: 1000,
        version: "0.1.0".to_string(),
    }
}
