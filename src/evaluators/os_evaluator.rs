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
    use crate::gatekeeper::test_helper;
    use anyhow::Result;

    #[cfg(target_os = "linux")]
    const OS : &str = "linux";
    #[cfg(target_os = "macos")]
    const OS : &str = "macos";
    #[cfg(target_os = "windows")]
    const OS : &str = "windows";

    #[test]
    fn test_pass() -> Result<()> {
        let os = format!("os_{}_pass", OS);
        test_helper(&os, true)
    }

    #[test]
    fn test_fail() -> Result<()> {
        match OS {
            "linux" => test_helper("os_macos_pass", false),
            "macos" => test_helper("os_windows_pass", false),
            "windows" => test_helper("os_linux_pass", false),
            _ => panic!("Unknown OS"),
        }
    }

    #[test]
    fn test_unix() -> Result<()> {
        match OS {
            "linux" => test_helper("os_unix_pass", true),
            "macos" => test_helper("os_unix_pass", true),
            "windows" => test_helper("os_unix_pass", false),
            _ => panic!("Unknown OS"),
        }
    }
}
