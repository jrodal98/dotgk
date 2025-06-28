use serde::Deserialize;
use serde::Serialize;

use super::EvaluatorTrait;

#[derive(Serialize, Deserialize, Debug)]
pub struct HostnameEvaluator {
    target: String,
}

impl EvaluatorTrait for HostnameEvaluator {
    fn evaluate(&self) -> bool {
        self.target == hostname::get().unwrap().to_str().unwrap().to_string()
    }
}
