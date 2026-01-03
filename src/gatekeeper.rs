use anyhow::Context;
use anyhow::Result;

use crate::lua_executor::LuaExecutor;

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

#[derive(Debug, Clone)]
pub struct GatekeeperResult {
    pub value: bool,
    pub ttl: Option<u64>,
}

pub fn load_and_evaluate_gatekeeper(name: &str) -> Result<GatekeeperResult> {
    // Auto-detect if we're loading an init.lua file and extract parent directory
    let gatekeeper_path = get_gatekeeper_path(name)?;

    // Check if the resolved path is an init.lua file
    let current_dir = if gatekeeper_path.ends_with("init.lua") {
        // This is an init.lua file - extract parent directory
        // E.g., /path/to/meta/init.lua -> "meta"
        gatekeeper_path
            .parent()  // Get directory containing init.lua
            .and_then(|p| p.file_name())  // Get directory name
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
    } else {
        None
    };

    load_and_evaluate_gatekeeper_with_context(name, current_dir)
}

pub fn load_and_evaluate_gatekeeper_with_context(name: &str, current_dir: Option<String>) -> Result<GatekeeperResult> {
    let gatekeeper_path = get_gatekeeper_path(name)
        .with_context(|| format!("Failed to get gatekeeper path for '{}'", name))?;

    if !gatekeeper_path.exists() {
        anyhow::bail!("Gatekeeper '{}' not found at {:?}", name, gatekeeper_path);
    }

    let script = std::fs::read_to_string(&gatekeeper_path).with_context(|| {
        format!(
            "Failed to read gatekeeper '{}' at path '{}'",
            name,
            gatekeeper_path.display()
        )
    })?;

    let executor = LuaExecutor::new()
        .context("Failed to create Lua executor")?;

    // Set current directory context if provided (for init.lua files)
    if let Some(dir) = current_dir {
        executor.set_current_dir(&dir)?;
    }

    let result = executor.execute(&script)
        .with_context(|| format!("Failed to execute Lua gatekeeper '{}'", name))?;

    Ok(GatekeeperResult {
        value: result.value,
        ttl: result.ttl,
    })
}

#[cfg(test)]
pub fn test_helper(name: &str, expected: bool) -> Result<()> {
    let result = load_and_evaluate_gatekeeper(name)?;
    assert_eq!(result.value, expected);
    Ok(())
}

pub fn get_gatekeeper_path(name: &str) -> Result<std::path::PathBuf> {
    let mut config_dir = get_config_dir()?;
    config_dir.push("gatekeepers");

    // For names without '/', try both direct file and init.lua
    if !name.contains('/') {
        // First try direct file (e.g., "server.lua")
        let direct_path = config_dir.join(format!("{}.lua", name));
        if direct_path.exists() {
            return Ok(direct_path);
        }

        // Then try init.lua (e.g., "myapp/init.lua")
        let init_path = config_dir.join(name).join("init.lua");
        if init_path.exists() {
            return Ok(init_path);
        }

        // Return direct path for error message consistency
        Ok(direct_path)
    } else {
        // For paths with '/', just use direct approach
        config_dir.push(format!("{}.lua", name));
        Ok(config_dir)
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

        if path.is_file() && path.extension().map_or(false, |ext| ext == "lua") {
            if let Some(stem) = path.file_stem() {
                if let Some(name) = stem.to_str() {
                    // Skip init.lua files - they're accessed as directory aggregates
                    if name == "init" {
                        continue;
                    }

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
                    // Check if this directory has an init.lua
                    let init_path = path.join("init.lua");
                    if init_path.exists() {
                        // Add the directory itself as a gatekeeper
                        let dir_gk_name = if prefix.is_empty() {
                            dir_str.to_string()
                        } else {
                            format!("{}/{}", prefix, dir_str)
                        };
                        gatekeepers.push(dir_gk_name);
                    }

                    // Recurse into subdirectory
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

    // Test subdirectory-based gatekeeper loading
    #[test]
    fn test_subdirectory_gatekeeper_myapp_devserver() -> Result<()> {
        test_helper("myapp/devserver", true)
    }

    #[test]
    fn test_subdirectory_gatekeeper_myapp_laptop() -> Result<()> {
        test_helper("myapp/laptop", false)
    }

    #[test]
    fn test_subdirectory_gatekeeper_os_linux() -> Result<()> {
        let expected = cfg!(target_os = "linux");
        test_helper("os/linux", expected)
    }

    #[test]
    fn test_subdirectory_composite_gatekeeper() -> Result<()> {
        test_helper("myapp/composite", true)
    }

    // Test basic evaluator types
    #[test]
    fn test_bool_pass() -> Result<()> {
        test_helper("bool_pass", true)
    }

    #[test]
    fn test_bool_fail() -> Result<()> {
        test_helper("bool_fail", false)
    }

    #[test]
    fn test_file_pass() -> Result<()> {
        test_helper("file_pass", true)
    }

    #[test]
    fn test_file_fail() -> Result<()> {
        test_helper("file_fail", false)
    }

    #[test]
    fn test_gatekeeper_pass() -> Result<()> {
        test_helper("gatekeeper_pass", true)
    }

    #[test]
    fn test_gatekeeper_fail() -> Result<()> {
        test_helper("gatekeeper_fail", false)
    }

    #[test]
    fn test_hostname_fail() -> Result<()> {
        test_helper("hostname_fail", false)
    }

    // Test OS-specific gatekeepers (portable via cfg!)
    #[test]
    fn test_os_linux_pass() -> Result<()> {
        let expected = cfg!(target_os = "linux");
        test_helper("os_linux_pass", expected)
    }

    #[test]
    fn test_os_macos_pass() -> Result<()> {
        let expected = cfg!(target_os = "macos");
        test_helper("os_macos_pass", expected)
    }

    #[test]
    fn test_os_unix_pass() -> Result<()> {
        let expected = cfg!(unix);
        test_helper("os_unix_pass", expected)
    }

    // Test find_all_gatekeepers includes subdirectory gatekeepers
    #[test]
    fn test_find_all_gatekeepers_includes_subdirectories() -> Result<()> {
        let gatekeepers = find_all_gatekeepers()?;

        // Should include both flat and subdirectory gatekeepers
        assert!(gatekeepers.contains(&"myapp/devserver".to_string()));
        assert!(gatekeepers.contains(&"myapp/laptop".to_string()));
        assert!(gatekeepers.contains(&"os/linux".to_string()));
        assert!(gatekeepers.contains(&"myapp/composite".to_string()));

        // Verify we have some subdirectory gatekeepers
        let subdir_gatekeepers: Vec<_> = gatekeepers
            .iter()
            .filter(|name| name.contains('/'))
            .collect();
        assert!(!subdir_gatekeepers.is_empty());

        Ok(())
    }
}
