use super::GroupEvaluator;

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct FileEvaluator;


// Implement GroupEvaluator for FileEvaluator
impl GroupEvaluator<String> for FileEvaluator {
    fn match_condition(&self, value: String) -> bool {
        PathBuf::from(value).exists()
    }
}
