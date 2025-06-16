use crate::gatekeeper::{Group, Evaluator};
use std::path::PathBuf;
use anyhow::Context;
use tracing::{debug, error, info, instrument};

pub trait GroupEvaluator {
    fn evaluate(&self, group: &Group) -> bool;
}

#[derive(Debug)]
pub struct HostnameEvaluator;
#[derive(Debug)]
pub struct FileEvaluator;

impl GroupEvaluator for HostnameEvaluator {
    #[instrument]
    fn evaluate(&self, group: &Group) -> bool {
        let hostname = hostname::get().context("Failed to get hostname").unwrap().into_string().unwrap();
        match group.condition_type.as_str() {
            "equal" => hostname == group.value,
            "not equal" => hostname != group.value,
            "one of" => {
                let values: Vec<&str> = group.value.split(',').map(|s| s.trim()).collect();
                values.contains(&hostname.as_str())
            }
            _ => {
                debug!("Invalid condition for hostname: {}", group.condition_type);
                false
            }
        }
    }
}

impl GroupEvaluator for FileEvaluator {
    fn evaluate(&self, group: &Group) -> bool {
        match group.condition_type.as_str() {
            "exists" => PathBuf::from(&group.value).exists(),
            "not exists" => !PathBuf::from(&group.value).exists(),
            _ => {
                eprintln!("Invalid condition for file: {}", group.condition_type);
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
