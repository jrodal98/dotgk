use std::path::PathBuf;

use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

use super::EvaluatorTrait;

#[derive(Serialize, Deserialize, Debug)]
pub struct FileEvaluator {
    path: String,
}

impl EvaluatorTrait for FileEvaluator {
    fn evaluate(&self) -> Result<bool> {
        Ok(PathBuf::from(&self.path).exists())
    }
}
