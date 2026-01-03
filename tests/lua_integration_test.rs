use anyhow::Result;
use dotgk::lua_executor::LuaExecutor;

#[test]
fn test_lua_simple_boolean() -> Result<()> {
    let executor = LuaExecutor::new()?;
    let result = executor.execute("return true")?;
    assert_eq!(result.value, true);
    assert_eq!(result.ttl, None);
    Ok(())
}

#[test]
fn test_lua_file_exists() -> Result<()> {
    let executor = LuaExecutor::new()?;

    // Test with a file that should exist on Linux
    if cfg!(target_os = "linux") {
        let result = executor.execute(r#"return file_exists("/etc/passwd")"#)?;
        assert_eq!(result.value, true);
    }

    // Test with a file that definitely doesn't exist
    let result = executor.execute(r#"return file_exists("/nonexistent/test/file/12345")"#)?;
    assert_eq!(result.value, false);
    Ok(())
}

#[test]
fn test_lua_os_check() -> Result<()> {
    let executor = LuaExecutor::new()?;

    let result = executor.execute(r#"return os("linux")"#)?;
    assert_eq!(result.value, cfg!(target_os = "linux"));

    let result = executor.execute(r#"return os("macos")"#)?;
    assert_eq!(result.value, cfg!(target_os = "macos"));

    let result = executor.execute(r#"return os("windows")"#)?;
    assert_eq!(result.value, cfg!(target_os = "windows"));

    Ok(())
}

#[test]
fn test_lua_any_combinator() -> Result<()> {
    let executor = LuaExecutor::new()?;

    // At least one true -> true
    let result = executor.execute(r#"return any({true, false, false})"#)?;
    assert_eq!(result.value, true);

    // All false -> false
    let result = executor.execute(r#"return any({false, false, false})"#)?;
    assert_eq!(result.value, false);

    // All true -> true
    let result = executor.execute(r#"return any({true, true, true})"#)?;
    assert_eq!(result.value, true);

    Ok(())
}

#[test]
fn test_lua_all_combinator() -> Result<()> {
    let executor = LuaExecutor::new()?;

    // All true -> true
    let result = executor.execute(r#"return all({true, true, true})"#)?;
    assert_eq!(result.value, true);

    // At least one false -> false
    let result = executor.execute(r#"return all({true, false, true})"#)?;
    assert_eq!(result.value, false);

    // All false -> false
    let result = executor.execute(r#"return all({false, false, false})"#)?;
    assert_eq!(result.value, false);

    Ok(())
}

#[test]
fn test_lua_none_combinator() -> Result<()> {
    let executor = LuaExecutor::new()?;

    // All false -> true (none are true)
    let result = executor.execute(r#"return none({false, false, false})"#)?;
    assert_eq!(result.value, true);

    // At least one true -> false
    let result = executor.execute(r#"return none({false, true, false})"#)?;
    assert_eq!(result.value, false);

    // All true -> false
    let result = executor.execute(r#"return none({true, true, true})"#)?;
    assert_eq!(result.value, false);

    Ok(())
}

#[test]
fn test_lua_variables() -> Result<()> {
    let executor = LuaExecutor::new()?;

    let result = executor.execute(r#"
        local is_linux = os("linux")
        local is_unix = os("unix")
        return is_linux or is_unix
    "#)?;

    if cfg!(unix) {
        assert_eq!(result.value, true);
    }

    Ok(())
}

#[test]
fn test_lua_ttl_comment() -> Result<()> {
    let executor = LuaExecutor::new()?;

    let result = executor.execute(r#"
        -- ttl: 3600
        return true
    "#)?;

    assert_eq!(result.value, true);
    assert_eq!(result.ttl, Some(3600));

    Ok(())
}

#[test]
fn test_lua_ttl_table() -> Result<()> {
    let executor = LuaExecutor::new()?;

    let result = executor.execute(r#"
        return {
            value = false,
            ttl = 7200,
        }
    "#)?;

    assert_eq!(result.value, false);
    assert_eq!(result.ttl, Some(7200));

    Ok(())
}

#[test]
fn test_lua_complex_logic() -> Result<()> {
    let executor = LuaExecutor::new()?;

    let result = executor.execute(r#"
        local is_unix = os("unix")
        local has_etc = file_exists("/etc")

        return all({
            is_unix,
            has_etc or os("windows"),
        })
    "#)?;

    // On Unix systems with /etc, this should be true
    if cfg!(unix) {
        assert_eq!(result.value, true);
    }

    Ok(())
}

#[test]
fn test_lua_syntax_error() {
    let executor = LuaExecutor::new().unwrap();

    let result = executor.execute("return invalid syntax here");
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Lua execution failed"));
}

#[test]
fn test_lua_wrong_return_type() {
    let executor = LuaExecutor::new().unwrap();

    let result = executor.execute(r#"return "string value""#);
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("must return a boolean"));
}
