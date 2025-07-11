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
    config_dir.push("gatekeepers");

    // Check if name contains a subdirectory (e.g., "meta/devserver")
    if name.contains('/') {
        config_dir.push(format!("{}.json", name));
    } else {
        config_dir.push(format!("{}.json", name));
    }

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

        let gatekeeper_content = std::fs::read_to_string(&gatekeeper_path).with_context(|| {
            format!(
                "Failed to read gatekeeper '{}' at path '{}'",
                name,
                gatekeeper_path.display()
            )
        })?;

        let gatekeeper = Self::from_json(&gatekeeper_content)
            .with_context(|| format!("Failed to parse gatekeeper '{}'", name))?;

        Ok(gatekeeper)
    }
}

pub fn find_all_gatekeepers() -> Result<Vec<String>> {
    let mut config_dir = get_config_dir()?;
    config_dir.push("gatekeepers");

    if !config_dir.exists() {
        return Ok(Vec::new());
    }

    let mut gatekeepers = Vec::new();
    find_gatekeepers_recursive(&config_dir, "", &mut gatekeepers)?;
    Ok(gatekeepers)
}

fn find_gatekeepers_recursive(
    dir: &std::path::Path,
    prefix: &str,
    gatekeepers: &mut Vec<String>,
) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
            if let Some(stem) = path.file_stem() {
                if let Some(name) = stem.to_str() {
                    let full_name = if prefix.is_empty() {
                        name.to_string()
                    } else {
                        format!("{}/{}", prefix, name)
                    };
                    gatekeepers.push(full_name);
                }
            }
        } else if path.is_dir() {
            if let Some(dir_name) = path.file_name() {
                if let Some(dir_str) = dir_name.to_str() {
                    let new_prefix = if prefix.is_empty() {
                        dir_str.to_string()
                    } else {
                        format!("{}/{}", prefix, dir_str)
                    };
                    find_gatekeepers_recursive(&path, &new_prefix, gatekeepers)?;
                }
            }
        }
    }
    Ok(())
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

    // Test subdirectory-based gatekeeper loading
    #[test]
    fn test_subdirectory_gatekeeper_meta_devserver() -> Result<()> {
        test_helper("meta/devserver", true)
    }

    #[test]
    fn test_subdirectory_gatekeeper_meta_laptop() -> Result<()> {
        test_helper("meta/laptop", false)
    }

    #[test]
    fn test_subdirectory_gatekeeper_os_linux() -> Result<()> {
        test_helper("os/linux", true)
    }

    #[test]
    fn test_subdirectory_composite_gatekeeper() -> Result<()> {
        test_helper("meta/composite", true)
    }

    // Test find_all_gatekeepers includes subdirectory gatekeepers
    #[test]
    fn test_find_all_gatekeepers_includes_subdirectories() -> Result<()> {
        let gatekeepers = find_all_gatekeepers()?;

        // Should include both flat and subdirectory gatekeepers
        assert!(gatekeepers.contains(&"meta/devserver".to_string()));
        assert!(gatekeepers.contains(&"meta/laptop".to_string()));
        assert!(gatekeepers.contains(&"os/linux".to_string()));
        assert!(gatekeepers.contains(&"meta/composite".to_string()));

        // Verify we have some subdirectory gatekeepers
        let subdir_gatekeepers: Vec<_> = gatekeepers
            .iter()
            .filter(|name| name.contains('/'))
            .collect();
        assert!(!subdir_gatekeepers.is_empty());

        Ok(())
    }
}
