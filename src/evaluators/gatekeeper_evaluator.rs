use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

use super::EvaluatorTrait;
use crate::gatekeeper::Gatekeeper;
use crate::gatekeeper::evaluate_gatekeeper;
use crate::gatekeeper::get_gatekeeper_path;

#[derive(Serialize, Deserialize, Debug)]
pub struct GatekeeperEvaluator {
    name: String,
}

impl EvaluatorTrait for GatekeeperEvaluator {
    fn evaluate(&self) -> Result<bool> {
        // Get the path to the referenced gatekeeper
        let gatekeeper_path = get_gatekeeper_path(&self.name)
            .with_context(|| format!("Failed to get path for gatekeeper '{}'", self.name))?;

        // Check if the gatekeeper exists
        if !gatekeeper_path.exists() {
            return Err(anyhow::anyhow!(
                "Referenced gatekeeper '{}' not found at {:?}",
                self.name,
                gatekeeper_path
            ));
        }

        // Read the gatekeeper content
        let gatekeeper_content = std::fs::read_to_string(&gatekeeper_path)
            .with_context(|| format!("Failed to read referenced gatekeeper '{}'", self.name))?;

        // Parse the gatekeeper
        let gatekeeper: Gatekeeper = serde_json::from_str(&gatekeeper_content)
            .with_context(|| format!("Failed to parse referenced gatekeeper '{}'", self.name))?;

        // Evaluate the gatekeeper
        evaluate_gatekeeper(&gatekeeper)
            .with_context(|| format!("Failed to evaluate referenced gatekeeper '{}'", self.name))
    }
}
