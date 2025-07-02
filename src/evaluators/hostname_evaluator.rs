use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

use super::EvaluatorTrait;

#[derive(Serialize, Deserialize, Debug)]
pub struct HostnameEvaluator {
    target: String,
}

impl EvaluatorTrait for HostnameEvaluator {
    fn evaluate(&self) -> Result<bool> {
        #[cfg(test)]
        {
            let hostname_str = "test-hostname";
            return Ok(self.target == hostname_str);
        }

        let hostname = hostname::get().context("Failed to get hostname")?;
        let hostname_str = hostname
            .to_str()
            .context("Failed to convert hostname to string")?;
        Ok(self.target == hostname_str)
    }
}


#[cfg(test)]
mod tests {
    use crate::gatekeeper::Gatekeeper;
    use anyhow::Result;

    fn make_hostname_gatekeeper_json(target: &str) -> String {
        serde_json::json!({
            "groups": [
                {
                    "type": "hostname",
                    "args": {
                        "target": target
                    },
                    "condition": "eq"
                }
            ]
        }).to_string()
    }

    #[test]
    fn test_pass() -> Result<()> {
        let json = make_hostname_gatekeeper_json("test-hostname");
        let result = Gatekeeper::evaluate_from_json(&json)?;
        assert!(result);
        Ok(())
    }

    #[test]
    fn test_fail() -> Result<()> {
        let json = make_hostname_gatekeeper_json("not-the-hostname");
        let result = Gatekeeper::evaluate_from_json(&json)?;
        assert!(!result);
        Ok(())
    }
}
