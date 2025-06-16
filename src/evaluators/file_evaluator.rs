use super::GroupEvaluator;
use super::ConditionType;
use super::Evaluator;

use std::path::PathBuf;

#[derive(Debug)]
pub struct FileEvaluator;

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
