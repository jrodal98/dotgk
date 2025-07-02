mod file_evaluator;
mod gatekeeper_evaluator;
mod hostname_evaluator;
mod os_evaluator;
mod evaluator_type;

use anyhow::Result;
use evaluator_type::EvaluatorType;
use serde::Deserialize;
use serde::Serialize;

// Define a trait for evaluators
pub trait EvaluatorTrait {
    fn evaluate(&self) -> Result<bool>;
}

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
    #[serde(flatten)]
    pub evaluator_type: EvaluatorType,
    pub condition: ConditionType,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl<T: EvaluatorTrait> OneOrMany<T> {
    fn match_eq(&self) -> Result<bool> {
        let results: Result<Vec<_>> = self.iter().map(|v| v.evaluate()).collect();
        Ok(results?.iter().all(|&result| result))
    }

    fn match_neq(&self) -> Result<bool> {
        let results: Result<Vec<_>> = self.iter().map(|v| v.evaluate()).collect();
        Ok(results?.iter().all(|&result| !result))
    }

    fn match_any(&self) -> Result<bool> {
        let results: Result<Vec<_>> = self.iter().map(|v| v.evaluate()).collect();
        Ok(results?.iter().any(|&result| result))
    }

    fn match_all(&self) -> Result<bool> {
        let results: Result<Vec<_>> = self.iter().map(|v| v.evaluate()).collect();
        Ok(results?.iter().all(|&result| result))
    }

    fn match_none(&self) -> Result<bool> {
        let results: Result<Vec<_>> = self.iter().map(|v| v.evaluate()).collect();
        Ok(results?.iter().all(|&result| !result))
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        match self {
            OneOrMany::One(v) => Box::new(std::iter::once(v)),
            OneOrMany::Many(v) => Box::new(v.iter()),
        }
    }
}

impl<T> IntoIterator for OneOrMany<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::One(v) => vec![v].into_iter(),
            Self::Many(v) => v.into_iter(),
        }
    }
}

impl Evaluator {
    pub fn evaluate(&self) -> Result<bool> {
        match &self.condition {
            ConditionType::Eq => self.evaluator_type.match_eq(),
            ConditionType::Neq => self.evaluator_type.match_neq(),
            ConditionType::Any => self.evaluator_type.match_any(),
            ConditionType::All => self.evaluator_type.match_all(),
            ConditionType::None => self.evaluator_type.match_none(),
        }
    }
}
