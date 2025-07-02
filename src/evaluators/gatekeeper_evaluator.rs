use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

use super::EvaluatorTrait;
use crate::gatekeeper::Gatekeeper;

#[derive(Serialize, Deserialize, Debug)]
pub struct GatekeeperEvaluator {
    name: String,
}

impl EvaluatorTrait for GatekeeperEvaluator {
    fn evaluate(&self) -> Result<bool> {
        let gk = Gatekeeper::from_name(&self.name)?;
        gk.evaluate()
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
                    "type": "gatekeeper",
                    "args": {
                        "name": target
                    },
                    "condition": "eq"
                }
            ]
        }).to_string();
        Gatekeeper::from_json(&gk_json)
    }

    fn helper(gk: &str, expected: bool) -> Result<()> {
        let gk = get_gk(gk)?;
        let result = gk.evaluate()?;
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_pass() -> Result<()> {
        helper("hostname_pass", true)
    }

    // #[test]
    // fn test_fail() -> Result<()> {
    //     helper("not-the-right-os", false)
    // }
}
