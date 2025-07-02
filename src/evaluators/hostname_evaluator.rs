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
        let hostname = hostname::get().context("Failed to get hostname")?;
        let hostname_str = hostname
            .to_str()
            .context("Failed to convert hostname to string")?;
        Ok(self.target == hostname_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gatekeeper::Gatekeeper;

    #[test]
    fn test_pass() -> Result<()> {
        // Example JSON for a hostname gatekeeper
        let json = format!(r#"
        {{
            "groups": [
                {{
                    "type": "hostname",
                    "args": {{
                        "target": "{}"
                    }},
                    "condition": "eq"
                }}
            ]
        }}
        "#, hostname::get().context("Failed to get hostname")?.to_str().context("Failed to convert hostname to string")?);

        let result = Gatekeeper::evaluate_from_json(&json)?;

        assert!(result);
        Ok(())
    }

    #[test]
    fn test_fail() -> Result<()> {
        // Example JSON for a hostname gatekeeper
        let json = format!(r#"
        {{
            "groups": [
                {{
                    "type": "hostname",
                    "args": {{
                        "target": "{}"
                    }},
                    "condition": "eq"
                }}
            ]
        }}
        "#, "hopefullynotarealhostname");

        let result = Gatekeeper::evaluate_from_json(&json)?;

        assert!(!result);
        Ok(())
    }
}
