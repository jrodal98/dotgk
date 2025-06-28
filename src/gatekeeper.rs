use std::env;

use anyhow::Context;
use anyhow::Result;
use dirs::config_dir;
use serde::Deserialize;
use serde::Serialize;

use crate::evaluators::Evaluator;

#[derive(Serialize, Deserialize, Debug)]
pub struct Gatekeeper {
    pub groups: Vec<Group>,
    #[serde(default = "default_false")]
    pub on_no_match: bool,
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
    } else if let Ok(env_path) = env::var("DOTGK_CONFIG_DIR") {
        std::path::PathBuf::from(env_path)
    } else {
        config_dir().context("Failed to get config directory")?
    };
    config_dir.push("dotgk");
    config_dir.push(format!("{}.json", name));
    Ok(config_dir)
}

pub fn evaluate_gatekeeper(gatekeeper: &Gatekeeper) -> Result<bool> {
    for group in gatekeeper.groups.iter() {
        let is_match = group.evaluator.evaluate()?;
        match (is_match, group.on_match) {
            (true, true) => return Ok(true),
            (true, false) => return Ok(false),
            (false, _) => continue,
        }
    }
    Ok(gatekeeper.on_no_match)
}
