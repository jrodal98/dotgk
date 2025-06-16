use crate::gatekeeper::{Group, ConditionType, Evaluator};
use std::path::PathBuf;
use anyhow::Context;
use tracing::{debug, instrument};

pub trait GroupEvaluator {
    fn evaluate(&self, group: &Group) -> bool;
}

#[derive(Debug)]
pub struct HostnameEvaluator;
#[derive(Debug)]
pub struct FileEvaluator;

// Implement GroupEvaluator for HostnameEvaluator
impl GroupEvaluator for HostnameEvaluator {
    #[instrument]
    fn evaluate(&self, group: &Group) -> bool {
        let hostname = hostname::get().context("Failed to get hostname").unwrap().into_string().unwrap();
        match &group.condition_type {
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
            _ => {
                debug!("Invalid condition type for hostname evaluator");
                false
            }
        }
    }
}

// Implement GroupEvaluator for FileEvaluator
impl GroupEvaluator for FileEvaluator {
    fn evaluate(&self, group: &Group) -> bool {
        match &group.condition_type {
            ConditionType::Exists => PathBuf::from(&group.value.as_str().unwrap()).exists(),
            ConditionType::NotExists => !PathBuf::from(&group.value.as_str().unwrap()).exists(),
            _ => {
                eprintln!("Invalid condition type for file evaluator");
                false
            }
        }
    }
}

impl GroupEvaluator for Evaluator {
    #[instrument]
    fn evaluate(&self, group: &Group) -> bool {
        match self {
            Evaluator::Hostname => HostnameEvaluator.evaluate(group),
            Evaluator::File => FileEvaluator.evaluate(group),
        }
    }
}
