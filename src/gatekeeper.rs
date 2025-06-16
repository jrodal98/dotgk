use crate::evaluators::GroupEvaluator;
use anyhow::Context;
use anyhow::Result;
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct Gatekeeper {
    pub groups: Vec<Group>,
    #[serde(default = "default_false")]
    pub on_no_match: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ConditionType {
    Eq,
    Neq,
    Any,
    All,
    None,
    //
    Exists,
    NotExists,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Group {
    pub evaluator: Evaluator,
    #[serde(rename = "condition")]
    pub condition_type: ConditionType,
    pub value: Value,
    #[serde(default = "default_true")]
    pub on_match: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Evaluator {
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
    for group in gatekeeper.groups.iter() {
        let is_match = group.evaluator.evaluate(group);
        match (is_match, group.on_match) {
            (true, true) => return true,
            (true, false) => return false,
            (false, _) => continue
        }

    }
    gatekeeper.on_no_match
}
