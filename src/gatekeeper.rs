use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

use crate::evaluators::ConditionType;
use crate::evaluators::Evaluator;

#[cfg(not(test))]
pub fn get_config_dir() -> Result<std::path::PathBuf> {
    if let Ok(env_path) = std::env::var("DOTGK_CONFIG_DIR") {
        Ok(std::path::PathBuf::from(env_path))
    } else {
        let home_dir = dirs::home_dir().context("Failed to get home directory")?;
        Ok(home_dir.join(".config").join("dotgk"))
    }
}

#[cfg(test)]
pub fn get_config_dir() -> Result<std::path::PathBuf> {
    Ok(std::path::PathBuf::from("examples/dotgk"))
}

#[cfg(test)]
pub fn test_helper(name: &str, expected: bool) -> Result<()> {
    let gk = Gatekeeper::from_name(name)?;
    let result = gk.evaluate()?;
    assert_eq!(result, expected);
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Gatekeeper {
    pub groups: Vec<Group>,
    #[serde(default = "default_condition")]
    pub condition: ConditionType,
    /// Optional TTL in seconds for cache entries created from this gatekeeper
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Group {
    #[serde(flatten)]
    pub evaluator: Evaluator,
}

fn default_condition() -> ConditionType {
    ConditionType::Any
}

pub fn get_gatekeeper_path(name: &str) -> Result<std::path::PathBuf> {
    let mut config_dir = get_config_dir()?;
    config_dir.push(format!("{}.json", name));
    Ok(config_dir)
}

impl Gatekeeper {
    pub fn evaluate(&self) -> Result<bool> {
        match &self.condition {
            ConditionType::Any => self.evaluate_any(),
            ConditionType::All => self.evaluate_all(),
            ConditionType::None => self.evaluate_none(),
            ConditionType::Eq => self.evaluate_any(), // Treat Eq as Any for group-level evaluation
            ConditionType::Neq => self.evaluate_none(), // Treat Neq as None for group-level evaluation
        }
    }

    fn evaluate_any(&self) -> Result<bool> {
        // If any group matches, return true
        // If no groups match, return false
        for group in self.groups.iter() {
            if group.evaluator.evaluate()? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn evaluate_all(&self) -> Result<bool> {
        // All groups must match to return true
        // If any group doesn't match, return false
        if self.groups.is_empty() {
            return Ok(true); // Vacuous truth: all of zero groups match
        }

        for group in self.groups.iter() {
            if !group.evaluator.evaluate()? {
                return Ok(false);
            }
        }
        Ok(true) // All groups matched
    }

    fn evaluate_none(&self) -> Result<bool> {
        // No groups should match to return true
        // If any group matches, return false
        for group in self.groups.iter() {
            if group.evaluator.evaluate()? {
                return Ok(false); // A group matched, so "none" fails
            }
        }
        Ok(true) // No groups matched, so "none" succeeds
    }

    pub fn from_json(json: &str) -> Result<Gatekeeper> {
        let gatekeeper: Gatekeeper = serde_json::from_str(json)
            .with_context(|| format!("Failed to parse gatekeeper from json '{}'", json))?;
        Ok(gatekeeper)
    }

    pub fn from_name(name: &str) -> Result<Gatekeeper> {
        let gatekeeper_path = get_gatekeeper_path(name)
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

    if !config_dir.exists() {
        return Ok(Vec::new());
    }

    let mut gatekeepers = Vec::new();
    for entry in std::fs::read_dir(&config_dir)? {
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

#[cfg(test)]
mod tests {
    use super::*;

    // Test "any" condition - should pass if any group matches
    #[test]
    fn test_condition_any_pass() -> Result<()> {
        test_helper("condition_any_pass", true)
    }

    #[test]
    fn test_condition_any_pass_second() -> Result<()> {
        test_helper("condition_any_pass_second", true)
    }

    #[test]
    fn test_condition_any_fail() -> Result<()> {
        test_helper("condition_any_fail", false)
    }

    // Test "all" condition - should pass only if all groups match
    #[test]
    fn test_condition_all_pass() -> Result<()> {
        test_helper("condition_all_pass", true)
    }

    #[test]
    fn test_condition_all_fail() -> Result<()> {
        test_helper("condition_all_fail", false)
    }

    #[test]
    fn test_condition_all_fail_first() -> Result<()> {
        test_helper("condition_all_fail_first", false)
    }

    // Test "none" condition - should pass only if no groups match
    #[test]
    fn test_condition_none_pass() -> Result<()> {
        test_helper("condition_none_pass", true)
    }

    #[test]
    fn test_condition_none_fail() -> Result<()> {
        test_helper("condition_none_fail", false)
    }

    #[test]
    fn test_condition_none_fail_first() -> Result<()> {
        test_helper("condition_none_fail_first", false)
    }

    #[test]
    fn test_condition_none_fail_second() -> Result<()> {
        test_helper("condition_none_fail_second", false)
    }

    // Test that the default condition is Any
    #[test]
    fn test_default_condition_is_any() -> Result<()> {
        let default_condition = default_condition();
        assert!(matches!(default_condition, ConditionType::Any));
        Ok(())
    }

    // Test that condition defaults to "any" when not specified in JSON
    #[test]
    fn test_default_condition_from_json() -> Result<()> {
        let json = r#"{
            "groups": [
                {
                    "type": "bool",
                    "args": {"pass": true},
                    "condition": "eq"
                }
            ]
        }"#;

        let gatekeeper = Gatekeeper::from_json(json)?;
        assert!(matches!(gatekeeper.condition, ConditionType::Any));
        let result = gatekeeper.evaluate()?;
        assert_eq!(result, true);
        Ok(())
    }

    // Test "all" condition with no groups (vacuous truth)
    #[test]
    fn test_condition_all_empty_groups() -> Result<()> {
        let json = r#"{
            "condition": "all",
            "groups": []
        }"#;

        let gatekeeper = Gatekeeper::from_json(json)?;
        let result = gatekeeper.evaluate()?;
        assert_eq!(result, true); // Vacuous truth: all of zero groups match
        Ok(())
    }
}
