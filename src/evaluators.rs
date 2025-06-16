use crate::gatekeeper::{ConditionType, Evaluator, EvaluatorType};
use std::path::PathBuf;
use anyhow::Context;
use tracing::instrument;

pub trait GroupEvaluator {
    fn evaluate(&self, group: &Evaluator) -> bool;
}

#[derive(Debug)]
pub struct HostnameEvaluator;
#[derive(Debug)]
pub struct FileEvaluator;

// Implement GroupEvaluator for HostnameEvaluator
impl GroupEvaluator for HostnameEvaluator {
    #[instrument]
    fn evaluate(&self, group: &Evaluator) -> bool {
        let hostname = hostname::get().context("Failed to get hostname").unwrap().into_string().unwrap();
        match &group.condition {
            ConditionType::Eq => hostname == group.value.as_str().unwrap(),
            ConditionType::Neq => hostname != group.value.as_str().unwrap(),
            ConditionType::Any => {
                let values: Vec<&str> = group.value.as_array().unwrap().into_iter().map(|v| v.as_str().unwrap()).collect();
                values.contains(&hostname.as_str())
            }
            ConditionType::All => {
                let values: Vec<&str> = group.value.as_array().unwrap().into_iter().map(|v| v.as_str().unwrap()).collect();
                values.iter().all(|v| hostname == *v)
            }
            ConditionType::None => {
                let values: Vec<&str> = group.value.as_array().unwrap().into_iter().map(|v| v.as_str().unwrap()).collect();
                !values.contains(&hostname.as_str())
            }
        }
    }
}

// Implement GroupEvaluator for FileEvaluator
impl GroupEvaluator for FileEvaluator {
    fn evaluate(&self, group: &Evaluator) -> bool {
        match &group.condition {
            ConditionType::Eq => PathBuf::from(&group.value.as_str().unwrap()).exists(),
            ConditionType::Neq => !PathBuf::from(&group.value.as_str().unwrap()).exists(),
            _ => {
                eprintln!("Invalid condition type for file evaluator");
                false
            }
        }
    }
}

impl Evaluator {
    pub fn evaluate(&self) -> bool {
        match self.evaluator_type {
            EvaluatorType::Hostname => HostnameEvaluator.evaluate(self),
            EvaluatorType::File => FileEvaluator.evaluate(self),
        }

    }

}
