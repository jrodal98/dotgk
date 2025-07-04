use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use tracing::debug;

use crate::gatekeeper::get_config_dir;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    /// List of enabled cache formats to generate
    #[serde(default)]
    pub enabled_cache_formats: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled_cache_formats: Vec::new(),
        }
    }
}

pub fn get_settings_path() -> Result<PathBuf> {
    let mut config_dir = get_config_dir()?;
    config_dir.push("settings.json");
    Ok(config_dir)
}

pub fn load_settings() -> Result<Settings> {
    let settings_path = get_settings_path()?;

    if !settings_path.exists() {
        debug!(
            "Settings file not found at {:?}, using defaults",
            settings_path
        );
        return Ok(Settings::default());
    }

    let settings_content = fs::read_to_string(&settings_path)
        .with_context(|| format!("Failed to read settings file at {:?}", settings_path))?;

    let settings: Settings = serde_json::from_str(&settings_content).with_context(|| {
        format!(
            "Failed to parse settings file at {:?}. Content: '{}'",
            settings_path, settings_content
        )
    })?;

    debug!("Loaded settings from {:?}: {:?}", settings_path, settings);
    Ok(settings)
}

pub fn save_settings(settings: &Settings) -> Result<()> {
    let settings_path = get_settings_path()?;

    // Create parent directory if it doesn't exist
    if let Some(parent) = settings_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let settings_json =
        serde_json::to_string_pretty(settings).context("Failed to serialize settings")?;

    fs::write(&settings_path, settings_json)
        .with_context(|| format!("Failed to write settings to {:?}", settings_path))?;

    debug!("Saved settings to {:?}", settings_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert!(settings.enabled_cache_formats.is_empty());
    }

    #[test]
    fn test_settings_serialization() -> Result<()> {
        let settings = Settings {
            enabled_cache_formats: vec!["Lua".to_string(), "shell".to_string()],
        };

        let json = serde_json::to_string_pretty(&settings)?;
        let deserialized: Settings = serde_json::from_str(&json)?;

        assert_eq!(deserialized.enabled_cache_formats.len(), 2);
        assert!(
            deserialized
                .enabled_cache_formats
                .contains(&"Lua".to_string())
        );
        assert!(
            deserialized
                .enabled_cache_formats
                .contains(&"shell".to_string())
        );

        Ok(())
    }

    #[test]
    fn test_partial_settings_deserialization() -> Result<()> {
        // Test that missing fields use defaults
        let json = r#"{}"#;
        let settings: Settings = serde_json::from_str(json)?;

        assert!(settings.enabled_cache_formats.is_empty());

        // Test with enabled_cache_formats
        let json = r#"{"enabled_cache_formats": ["Lua", "python"]}"#;
        let settings: Settings = serde_json::from_str(json)?;

        assert_eq!(settings.enabled_cache_formats.len(), 2);
        assert!(settings.enabled_cache_formats.contains(&"Lua".to_string()));
        assert!(
            settings
                .enabled_cache_formats
                .contains(&"python".to_string())
        );

        Ok(())
    }
}
