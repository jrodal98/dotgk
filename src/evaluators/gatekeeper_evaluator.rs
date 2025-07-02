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
