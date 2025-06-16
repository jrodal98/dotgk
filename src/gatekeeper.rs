use crate::evaluators;
use anyhow::Context;
use anyhow::Result;
use dirs::config_dir;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Gatekeeper {
    pub groups: Vec<Group>,
    #[serde(default = "default_false")]
    pub on_no_match: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Group {
    #[serde(rename = "type")]
    pub group_type: GroupType,
    #[serde(rename = "condition")]
    pub condition_type: String,
    pub value: String,
    #[serde(default = "default_true")]
    pub on_match: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum GroupType {
    Hostname,
    File,
}

fn default_false() -> bool {
    false
}

fn default_true() -> bool {
    true
}

pub fn get_gatekeeper_path(name: &str) -> Result<std::path::PathBuf> {
    let mut config_dir = config_dir().context("Failed to get config directory")?;
    config_dir.push("dotgk");
    config_dir.push(format!("{}.json", name));
    Ok(config_dir)
}

pub fn evaluate_gatekeeper(gatekeeper: &Gatekeeper) -> bool {
    gatekeeper.groups.iter().any(|group| {
        let evaluator = evaluators::get_evaluator(&group.group_type);
        evaluator.evaluate(group) && group.on_match
    }) || gatekeeper.on_no_match
}
