use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

use super::EvaluatorTrait;

#[derive(Serialize, Deserialize, Debug)]
pub struct BoolEvaluator {
    pass: bool,
}

impl EvaluatorTrait for BoolEvaluator {
    fn evaluate(&self) -> Result<bool> {
        Ok(self.pass)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::gatekeeper::test_helper;

    #[test]
    fn test_pass() -> Result<()> {
        test_helper("bool_pass", true)
    }

    #[test]
    fn test_fail() -> Result<()> {
        test_helper("bool_fail", false)
    }
}
