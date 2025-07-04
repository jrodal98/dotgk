use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use tracing::info;

use crate::cache::cache::Cache;
use crate::gatekeeper::get_config_dir;

pub mod lua;
pub mod python;
pub mod shell;

#[cfg(test)]
pub mod test_utils;

pub use lua::LuaCacheGenerator;
pub use python::PythonCacheGenerator;
pub use shell::ShellCacheGenerator;

/// Trait for generating cache files in different formats
pub trait CacheGenerator {
    /// Returns the name/identifier of this cache format
    fn name(&self) -> &'static str;

    /// Returns the file extension for this cache format
    fn file_extension(&self) -> &'static str;

    /// Generates the cache content as a string
    fn generate_content(&self, cache: &Cache) -> Result<String>;

    /// Generates the cache file at the specified path
    fn generate_file(&self, cache: &Cache, cache_path: Option<PathBuf>) -> Result<()> {
        let mut config_dir = get_config_dir()?;
        config_dir.push("caches");
        config_dir.push(format!("dotgk.{}", self.file_extension()));

        let file_path = cache_path.unwrap_or(config_dir);

        // Create cache directory if it doesn't exist
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = self.generate_content(cache)?;

        fs::write(&file_path, content)
            .with_context(|| format!("Failed to write {} cache to {:?}", self.name(), file_path))?;

        info!("Generated {} cache at {:?}", self.name(), file_path);
        Ok(())
    }
}

/// Registry for managing cache generators
pub struct CacheGeneratorRegistry {
    generators: HashMap<String, Box<dyn CacheGenerator>>,
}

impl CacheGeneratorRegistry {
    /// Create a new registry with default generators
    pub fn new() -> Self {
        let mut registry = Self {
            generators: HashMap::new(),
        };

        // Register built-in generators
        registry.register(Box::new(LuaCacheGenerator));
        registry.register(Box::new(PythonCacheGenerator));
        registry.register(Box::new(ShellCacheGenerator));

        registry
    }

    /// Register a new cache generator
    pub fn register(&mut self, generator: Box<dyn CacheGenerator>) {
        self.generators
            .insert(generator.name().to_string(), generator);
    }

    /// Get a generator by name
    pub fn get(&self, name: &str) -> Option<&dyn CacheGenerator> {
        self.generators.get(name).map(|g| g.as_ref())
    }

    /// Get all available generator names
    #[cfg(test)]
    pub fn available_generators(&self) -> Vec<&str> {
        self.generators.keys().map(|s| s.as_str()).collect()
    }

    /// Generate cache files for the specified formats
    pub fn generate_caches(&self, cache: &Cache, enabled_formats: &[String]) -> Vec<String> {
        let mut generated_formats = Vec::new();

        for format in enabled_formats {
            if let Some(generator) = self.get(format) {
                if let Err(e) = generator.generate_file(cache, None) {
                    tracing::error!("Failed to generate {} cache: {}", format, e);
                } else {
                    generated_formats.push(format.clone());
                }
            } else {
                tracing::warn!("Unknown cache format requested: {}", format);
            }
        }

        generated_formats
    }
}

impl Default for CacheGeneratorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry() {
        let registry = CacheGeneratorRegistry::new();

        assert!(registry.get("lua").is_some());
        assert!(registry.get("shell").is_some());
        assert!(registry.get("python").is_some());
        assert!(registry.get("nonexistent").is_none());

        let available = registry.available_generators();
        assert!(available.contains(&"lua"));
        assert!(available.contains(&"shell"));
        assert!(available.contains(&"python"));
    }
}
