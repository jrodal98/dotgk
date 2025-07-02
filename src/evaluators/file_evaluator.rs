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
    use crate::gatekeeper::test_helper;
    use anyhow::Result;


    #[test]
    fn test_pass() -> Result<()> {
        test_helper("file_pass", true)
    }

    #[test]
    fn test_fail() -> Result<()> {
        test_helper("file_fail", false)
    }
}
