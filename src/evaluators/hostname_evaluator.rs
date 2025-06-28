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
