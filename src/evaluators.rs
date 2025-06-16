use crate::gatekeeper::{Group, GroupType};
use std::path::PathBuf;
use anyhow::Context;

pub trait GroupEvaluator {
    fn evaluate(&self, group: &Group) -> bool;
}

pub struct HostnameEvaluator;
pub struct FileEvaluator;

impl GroupEvaluator for HostnameEvaluator {
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
                eprintln!("Invalid condition for hostname: {}", group.condition_type);
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

pub fn get_evaluator(group_type: &GroupType) -> Box<dyn GroupEvaluator> {
    match group_type {
        GroupType::Hostname => Box::new(HostnameEvaluator),
        GroupType::File => Box::new(FileEvaluator),
    }
}
