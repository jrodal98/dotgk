use std::env;

use anyhow::Context;
use anyhow::Result;
use dirs::config_dir;
use serde::Deserialize;
use serde::Serialize;

use crate::evaluators::Evaluator;

#[cfg(not(test))]
pub fn get_config_dir() -> Result<std::path::PathBuf> {
    if let Ok(env_path) = env::var("DOTGK_CONFIG_DIR") {
        Ok(std::path::PathBuf::from(env_path))
    } else {
        config_dir().context("Failed to get config directory")
    }
}

#[cfg(test)]
pub fn get_config_dir() -> Result<std::path::PathBuf> {
    Ok(std::path::PathBuf::from("examples"))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Gatekeeper {
    pub groups: Vec<Group>,
    #[serde(default = "default_false")]
    pub on_no_match: bool,
    /// Optional TTL in seconds for cache entries created from this gatekeeper
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Group {
    #[serde(flatten)]
    pub evaluator: Evaluator,
    #[serde(default = "default_true")]
    pub on_match: bool,
}

fn default_false() -> bool {
    false
}

fn default_true() -> bool {
    true
}

pub fn get_gatekeeper_path(
    name: &str,
    config_path: Option<std::path::PathBuf>,
) -> Result<std::path::PathBuf> {
    let mut config_dir = if let Some(path) = config_path {
        path
    } else {
        get_config_dir()?
    };
    config_dir.push("dotgk");
    config_dir.push(format!("{}.json", name));
    Ok(config_dir)
}


impl Gatekeeper {

    pub fn evaluate(&self) -> Result<bool> {
        for group in self.groups.iter() {
            let is_match = group.evaluator.evaluate()?;
            match (is_match, group.on_match) {
                (true, true) => return Ok(true),
                (true, false) => return Ok(false),
                (false, _) => continue,
            }
        }
        Ok(self.on_no_match)
    }

    pub fn from_json(json: &str) -> Result<Gatekeeper> {
        let gatekeeper: Gatekeeper = serde_json::from_str(json)
            .with_context(|| format!("Failed to parse gatekeeper from json '{}'", json))?;
        Ok(gatekeeper)
    }

    pub fn from_name(name: &str) -> Result<Gatekeeper> {
        let gatekeeper_path = get_gatekeeper_path(name, None)
            .with_context(|| format!("Failed to get gatekeeper path for '{}'", name))?;

        if !gatekeeper_path.exists() {
            anyhow::bail!("Gatekeeper '{}' not found at {:?}", name, gatekeeper_path);
        }

        let gatekeeper_content = std::fs::read_to_string(&gatekeeper_path)
            .with_context(|| format!("Failed to read gatekeeper '{}'", name))?;

        let gatekeeper = Self::from_json(&gatekeeper_content)
            .with_context(|| format!("Failed to parse gatekeeper '{}'", name))?;

        Ok(gatekeeper)
    }
}


pub fn find_all_gatekeepers() -> Result<Vec<String>> {
    let config_dir = get_config_dir()?;
    let dotgk_dir = config_dir.join("dotgk");

    if !dotgk_dir.exists() {
        return Ok(Vec::new());
    }

    let mut gatekeepers = Vec::new();
    for entry in std::fs::read_dir(&dotgk_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
            if let Some(stem) = path.file_stem() {
                if let Some(name) = stem.to_str() {
                    if name != "cache" {
                        gatekeepers.push(name.to_string());
                    }
                }
            }
        }
    }
    Ok(gatekeepers)
}
