mod file_evaluator;
mod hostname_evaluator;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ConditionType {
    Eq,
    Neq,
    Any,
    All,
    None,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Evaluator {
    #[serde(rename = "name")]
    pub evaluator_type: EvaluatorType,
    pub condition: ConditionType,
    pub value: Value,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum EvaluatorType {
    Hostname,
    File,
}

pub trait GroupEvaluator {
    fn evaluate(&self, group: &Evaluator) -> bool;
}


impl Evaluator {
    pub fn evaluate(&self) -> bool {
        match self.evaluator_type {
            EvaluatorType::Hostname => crate::evaluators::hostname_evaluator::HostnameEvaluator.evaluate(self),
            EvaluatorType::File => crate::evaluators::file_evaluator::FileEvaluator.evaluate(self),
        }

    }

}
