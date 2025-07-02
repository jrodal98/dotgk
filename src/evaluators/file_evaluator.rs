use std::path::PathBuf;

use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

use super::EvaluatorTrait;

#[derive(Serialize, Deserialize, Debug)]
pub struct FileEvaluator {
    path: String,
}

impl EvaluatorTrait for FileEvaluator {
    fn evaluate(&self) -> Result<bool> {
        Ok(PathBuf::from(&self.path).exists())
    }
}

#[cfg(test)]
mod tests {
    use crate::gatekeeper::Gatekeeper;
    use anyhow::Result;

    fn get_gk(target: &str) -> Result<Gatekeeper> {
        let gk_json = serde_json::json!({
            "groups": [
                {
                    "type": "file",
                    "args": {
                        "path": target
                    },
                    "condition": "eq"
                }
            ]
        }).to_string();
        Gatekeeper::from_json(&gk_json)
    }

    fn helper(target: &str, expected: bool) -> Result<()> {
        let gk = get_gk(target)?;
        let result = gk.evaluate()?;
        assert_eq!(result, expected);
        Ok(())
    }


    #[test]
    fn test_pass() -> Result<()> {
        helper("src/evaluators/file_evaluator.rs", true)
    }

    #[test]
    fn test_fail() -> Result<()> {
        helper("src/evaluators/file_evaluator-dne.rs", false)
    }
}
