use super::GroupEvaluator;
use super::ConditionType;
use super::Evaluator;

use std::path::PathBuf;

#[derive(Debug)]
pub struct FileEvaluator;

// Implement GroupEvaluator for FileEvaluator
impl GroupEvaluator<String> for FileEvaluator {
    fn single_passes(&self, value: String) -> bool {
        PathBuf::from(value).exists()
    }
}
