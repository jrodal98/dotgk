use anyhow::Result;
use mlua::prelude::*;
use regex::Regex;
use std::cell::RefCell;
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct LuaGatekeeperResult {
    pub value: bool,
    pub ttl: Option<u64>,
}

/// Tracks visited gatekeepers to detect circular dependencies
#[derive(Default)]
struct EvaluationContext {
    visited: RefCell<HashSet<String>>,
}

impl EvaluationContext {
    fn visit(&self, name: &str) -> Result<()> {
        let mut visited = self.visited.borrow_mut();
        if visited.contains(name) {
            let chain: Vec<String> = visited.iter().cloned().collect();
            anyhow::bail!(
                "Circular dependency detected: gatekeeper '{}' references itself\nCall chain: {} → {}",
                name,
                chain.join(" → "),
                name
            );
        }
        visited.insert(name.to_string());
        Ok(())
    }

    fn leave(&self, name: &str) {
        self.visited.borrow_mut().remove(name);
    }
}

pub struct LuaExecutor {
    lua: Lua,
    _context: std::rc::Rc<EvaluationContext>,
}

impl LuaExecutor {
    pub fn new() -> Result<Self> {
        let lua = Lua::new();
        let context = std::rc::Rc::new(EvaluationContext::default());

        // Register DSL functions
        Self::register_functions(&lua, context.clone())?;

        Ok(Self { lua, _context: context })
    }

    fn register_functions(lua: &Lua, context: std::rc::Rc<EvaluationContext>) -> Result<()> {
        let globals = lua.globals();

        // file_exists(path: string) -> bool
        let file_exists = lua.create_function(|_, path: String| {
            Ok(Path::new(&path).exists())
        })?;
        globals.set("file_exists", file_exists)?;

        // hostname(target: string) -> bool
        let hostname_check = lua.create_function(|_, target: String| {
            let current = hostname::get()
                .map_err(|e| LuaError::RuntimeError(format!("Failed to get hostname: {}", e)))?;
            let current_str = current
                .to_str()
                .ok_or_else(|| LuaError::RuntimeError("Invalid hostname encoding".into()))?;
            Ok(current_str == target)
        })?;
        globals.set("hostname", hostname_check)?;

        // os(name: string) -> bool
        let os_check = lua.create_function(|_, name: String| {
            let matches = match name.as_str() {
                "linux" => cfg!(target_os = "linux"),
                "macos" | "darwin" => cfg!(target_os = "macos"),
                "windows" => cfg!(target_os = "windows"),
                "unix" => cfg!(unix),
                _ => {
                    return Err(LuaError::RuntimeError(format!(
                        "Unknown OS '{}'. Valid options: linux, macos, windows, unix",
                        name
                    )))
                }
            };
            Ok(matches)
        })?;
        globals.set("os", os_check)?;

        // Register custom require searcher for loading other gatekeepers
        Self::register_require_searcher(lua, context.clone())?;

        // any(checks: table) -> bool
        let any_check = lua.create_function(|_, checks: Vec<bool>| Ok(checks.iter().any(|&x| x)))?;
        globals.set("any", any_check)?;

        // all(checks: table) -> bool
        let all_check = lua.create_function(|_, checks: Vec<bool>| Ok(checks.iter().all(|&x| x)))?;
        globals.set("all", all_check)?;

        // none(checks: table) -> bool
        let none_check =
            lua.create_function(|_, checks: Vec<bool>| Ok(!checks.iter().any(|&x| x)))?;
        globals.set("none", none_check)?;

        // bool(value: bool) -> bool (identity function for clarity)
        let bool_check = lua.create_function(|_, value: bool| Ok(value))?;
        globals.set("bool", bool_check)?;

        Ok(())
    }

    fn register_require_searcher(lua: &Lua, context: std::rc::Rc<EvaluationContext>) -> Result<()> {
        // Get package.searchers table
        let package: LuaTable = lua.globals().get("package")
            .map_err(|e| anyhow::anyhow!("Failed to get package table: {}", e))?;
        let searchers: LuaTable = package.get("searchers")
            .map_err(|e| anyhow::anyhow!("Failed to get package.searchers: {}", e))?;

        // Create custom searcher function
        let custom_searcher = lua.create_function(move |lua_ctx, module_name: String| {
            // Convert "meta.devserver" -> "meta/devserver"
            let gk_name = module_name.replace('.', "/");

            // Determine paths to try:
            // For "meta" -> try ["meta", "meta/init"]
            // For "meta/devserver" -> try ["meta/devserver"]
            let paths_to_try: Vec<String> = if !gk_name.contains('/') {
                // Simple name like "meta" - try both direct and init.lua
                vec![gk_name.clone(), format!("{}/init", gk_name)]
            } else {
                // Nested name like "meta/devserver" - just try as-is
                vec![gk_name.clone()]
            };

            // Try each path
            for path in &paths_to_try {
                // Check if gatekeeper file exists
                if let Ok(gk_path) = crate::gatekeeper::get_gatekeeper_path(path) {
                    if gk_path.exists() {
                        // Found it! Check for circular dependency
                        context
                            .visit(path)
                            .map_err(|e| LuaError::RuntimeError(e.to_string()))?;

                        // Create loader function that will be called by require()
                        let path_clone = path.clone();
                        let context_clone = context.clone();

                        let loader = lua_ctx.create_function(move |_lua, _: ()| {
                            // Load and evaluate the gatekeeper
                            let result = match crate::gatekeeper::load_and_evaluate_gatekeeper(&path_clone) {
                                Ok(result) => {
                                    context_clone.leave(&path_clone);
                                    Ok(result.value)
                                }
                                Err(e) => {
                                    context_clone.leave(&path_clone);
                                    Err(LuaError::RuntimeError(format!(
                                        "Failed to load gatekeeper '{}': {}\nHint: Check that the gatekeeper exists and has valid syntax",
                                        path_clone, e
                                    )))
                                }
                            };

                            result
                        })?;

                        // Return the loader function
                        return Ok(loader);
                    }
                }
            }

            // Not found - return error message with paths tried
            Err(LuaError::RuntimeError(format!(
                "Gatekeeper '{}' not found (tried: {})",
                module_name,
                paths_to_try.join(".lua, ") + ".lua"
            )))
        })
        .map_err(|e| anyhow::anyhow!("Failed to create custom searcher: {}", e))?;

        // Insert at the beginning of searchers table (index 1)
        searchers.raw_insert(1, custom_searcher)
            .map_err(|e| anyhow::anyhow!("Failed to insert custom searcher: {}", e))?;

        Ok(())
    }

    /// Execute a Lua script and return the result
    pub fn execute(&self, script: &str) -> Result<LuaGatekeeperResult> {
        // Parse TTL from comment if present (-- ttl: 3600)
        let ttl = Self::parse_ttl_comment(script);

        // Execute the Lua script
        let result: LuaValue = self
            .lua
            .load(script)
            .eval()
            .map_err(|e| anyhow::anyhow!("Lua execution failed:\n{}\nError: {}", Self::format_script(script), e))?;

        // Extract result
        match result {
            // Simple boolean return
            LuaValue::Boolean(value) => Ok(LuaGatekeeperResult { value, ttl }),

            // Table with value and optional ttl
            LuaValue::Table(table) => {
                let value = table
                    .get::<_, bool>("value")
                    .map_err(|_| anyhow::anyhow!("Table must contain a 'value' field of type boolean"))?;
                let table_ttl = table.get::<_, Option<u64>>("ttl").ok().flatten();
                Ok(LuaGatekeeperResult {
                    value,
                    ttl: table_ttl.or(ttl),
                })
            }

            _ => anyhow::bail!(
                "Lua script must return a boolean or table with 'value' field.\nGot: {:?}",
                result
            ),
        }
    }

    /// Parse TTL from comment like: -- ttl: 3600
    fn parse_ttl_comment(script: &str) -> Option<u64> {
        let re = Regex::new(r"^--\s*ttl:\s*(\d+)").ok()?;
        for line in script.lines() {
            if let Some(captures) = re.captures(line.trim()) {
                if let Some(ttl_str) = captures.get(1) {
                    return ttl_str.as_str().parse::<u64>().ok();
                }
            }
        }
        None
    }

    /// Format script with line numbers for error messages
    fn format_script(script: &str) -> String {
        script
            .lines()
            .enumerate()
            .map(|(i, line)| format!("{:3} | {}", i + 1, line))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_exists() {
        let executor = LuaExecutor::new().unwrap();
        let result = executor
            .execute(r#"return file_exists("/etc/passwd")"#)
            .unwrap();
        // This file should exist on Linux systems
        if cfg!(target_os = "linux") {
            assert_eq!(result.value, true);
        }
    }

    #[test]
    fn test_file_not_exists() {
        let executor = LuaExecutor::new().unwrap();
        let result = executor
            .execute(r#"return file_exists("/nonexistent/path/12345")"#)
            .unwrap();
        assert_eq!(result.value, false);
    }

    #[test]
    fn test_os_check() {
        let executor = LuaExecutor::new().unwrap();
        let result = executor.execute(r#"return os("linux")"#).unwrap();
        assert_eq!(result.value, cfg!(target_os = "linux"));
    }

    #[test]
    fn test_any_combinator() {
        let executor = LuaExecutor::new().unwrap();
        let result = executor
            .execute(
                r#"
            return any({
                file_exists("/nonexistent1"),
                file_exists("/etc/passwd"),
            })
        "#,
            )
            .unwrap();
        if cfg!(target_os = "linux") {
            assert_eq!(result.value, true);
        }
    }

    #[test]
    fn test_all_combinator() {
        let executor = LuaExecutor::new().unwrap();
        let result = executor
            .execute(
                r#"
            return all({
                file_exists("/etc/passwd"),
                file_exists("/etc/passwd"),
            })
        "#,
            )
            .unwrap();
        if cfg!(target_os = "linux") {
            assert_eq!(result.value, true);
        }
    }

    #[test]
    fn test_none_combinator() {
        let executor = LuaExecutor::new().unwrap();
        let result = executor
            .execute(
                r#"
            return none({
                file_exists("/nonexistent1"),
                file_exists("/nonexistent2"),
            })
        "#,
            )
            .unwrap();
        assert_eq!(result.value, true);
    }

    #[test]
    fn test_ttl_parsing() {
        let executor = LuaExecutor::new().unwrap();
        let result = executor
            .execute(
                r#"
            -- ttl: 3600
            return true
        "#,
            )
            .unwrap();
        assert_eq!(result.value, true);
        assert_eq!(result.ttl, Some(3600));
    }

    #[test]
    fn test_table_return() {
        let executor = LuaExecutor::new().unwrap();
        let result = executor
            .execute(
                r#"
            return {
                value = true,
                ttl = 7200,
            }
        "#,
            )
            .unwrap();
        assert_eq!(result.value, true);
        assert_eq!(result.ttl, Some(7200));
    }

    #[test]
    fn test_variables() {
        let executor = LuaExecutor::new().unwrap();
        let result = executor
            .execute(
                r#"
            local is_linux = os("linux")
            local has_passwd = file_exists("/etc/passwd")
            return is_linux and has_passwd
        "#,
            )
            .unwrap();
        if cfg!(target_os = "linux") {
            assert_eq!(result.value, true);
        }
    }
}
