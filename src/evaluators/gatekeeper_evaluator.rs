use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

use super::EvaluatorTrait;
use crate::gatekeeper::evaluate_gatekeeper_by_name;

#[derive(Serialize, Deserialize, Debug)]
pub struct GatekeeperEvaluator {
    name: String,
}

impl EvaluatorTrait for GatekeeperEvaluator {
    fn evaluate(&self) -> Result<bool> {
        evaluate_gatekeeper_by_name(&self.name)
    }
}
