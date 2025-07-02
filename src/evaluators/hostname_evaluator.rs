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
        let hostname = if cfg!(test) {
            "test-hostname".into()
        } else {
            hostname::get().context("Failed to get hostname")?
        };

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

    fn get_gk(target: &str) -> Result<Gatekeeper> {
        let gk_json = serde_json::json!({
            "groups": [
                {
                    "type": "hostname",
                    "args": {
                        "target": target
                    },
                    "condition": "eq"
                }
            ]
        }).to_string();
        Gatekeeper::from_json(&gk_json)
    }

    #[test]
    fn test_pass() -> Result<()> {
        let gk = get_gk("test-hostname")?;
        let result = gk.evaluate()?;
        assert!(result);
        Ok(())
    }

    #[test]
    fn test_fail() -> Result<()> {
        let gk = get_gk("not-test-hostname")?;
        let result = gk.evaluate()?;
        assert!(!result);
        Ok(())
    }
}
