use super::GroupEvaluator;
use super::ConditionType;
use super::Evaluator;
use anyhow::Context;

use tracing::{debug, error, info, instrument};

#[derive(Debug)]
pub struct HostnameEvaluator;

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
