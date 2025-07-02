use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

use super::EvaluatorTrait;

#[derive(Serialize, Deserialize, Debug)]
pub struct OSEvaluator {
    target: String,
}

impl EvaluatorTrait for OSEvaluator {
    fn evaluate(&self) -> Result<bool> {
        // https://doc.rust-lang.org/std/env/consts/constant.OS.html
        let os = std::env::consts::OS;
        Ok(os == self.target)
    }
}

#[cfg(test)]
mod tests {
    use crate::gatekeeper::Gatekeeper;
    use anyhow::Result;

    #[cfg(target_os = "linux")]
    const OS : &str = "linux";
    #[cfg(target_os = "macos")]
    const OS : &str = "macos";
    #[cfg(target_os = "windows")]
    const OS : &str = "windows";

    fn get_gk(target: &str) -> Result<Gatekeeper> {
        let gk_json = serde_json::json!({
            "groups": [
                {
                    "type": "os",
                    "args": {
                        "target": target
                    },
                    "condition": "eq"
                }
            ]
        }).to_string();
        Gatekeeper::from_json(&gk_json)
    }

    fn helper(os: &str, expected: bool) -> Result<()> {
        let gk = get_gk(os)?;
        let result = gk.evaluate()?;
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_pass() -> Result<()> {
        helper(OS, true)
    }

    #[test]
    fn test_fail() -> Result<()> {
        helper("not-the-right-os", false)
    }
}
