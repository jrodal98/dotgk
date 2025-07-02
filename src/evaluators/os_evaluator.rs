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
    use super::*;

    #[test]
    fn test_os_evaluator() {
        let os = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else {
            return;
        };

        let evaluator = OSEvaluator {
            target: os.to_string(),
        };
        assert!(evaluator.evaluate().unwrap());
    }
}
