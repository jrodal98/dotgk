mod file_evaluator;
mod hostname_evaluator;

use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    #[serde(rename = "name")]
    pub evaluator_type: EvaluatorType,
    pub condition: ConditionType,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl<T> OneOrMany<T> {
    fn into_vec(self) -> Vec<T> {
        match self {
            Self::One(v) => vec![v],
            Self::Many(v) => v,
        }
    }
}

pub trait MatchEvaluator<T> {
    fn match_condition(&self, value: T) -> bool;

    fn match_eq(&self, value: OneOrMany<T>) -> bool {
        match value {
            OneOrMany::One(v) => self.match_condition(v),
            OneOrMany::Many(v) => self.match_all(OneOrMany::Many(v))
        }
    }

    fn match_neq(&self, value: OneOrMany<T>) -> bool {
        !self.match_eq(value)
    }

    fn match_any(&self, value: OneOrMany<T>) -> bool {
        let values = value.into_vec();
        values.into_iter().any(|v| self.match_condition(v))
    }

    fn match_all(&self, value: OneOrMany<T>) -> bool {
        let values = value.into_vec();
        values.into_iter().all(|v| self.match_condition(v))
    }

    fn match_none(&self, value: OneOrMany<T>) -> bool {
        !self.match_any(value)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase", tag = "type", content = "args")]
pub enum EvaluatorType {
    Hostname(OneOrMany<String>),
    File(OneOrMany<String>),
    // FileContent(OneOrMany<String, String>),
}


impl Evaluator {
    pub fn evaluate(&self) -> bool {
        match &self.condition {
            ConditionType::Eq => self.evaluator_type.match_condition(),
            ConditionType::Neq => !self.match_condition(),
            ConditionType::Any => {
                let values: Vec<T> = self.evaluator_type.value_as_vec();
                values.into_iter().any(|v| self.match_condition(v))
            }
            ConditionType::All => {
                let values: Vec<T> = group.value_as_vec();
                values.into_iter().all(|v| self.match_condition(v))
            }
            ConditionType::None => {
                let values: Vec<T> = group.value_as_vec();
                !values.into_iter().any(|v| self.match_condition(v))
            }
        }
    }
}

pub trait GroupEvaluator<T>
where
    T: for<'de> serde::Deserialize<'de>,
{
    fn evaluate(&self, group: &Evaluator) -> bool {
        match &group.condition {
            ConditionType::Eq => self.match_condition(group.value_as_single()),
            ConditionType::Neq => !self.match_condition(group.value_as_single()),
            ConditionType::Any => {
                let values: Vec<T> = group.value_as_vec();
                values.into_iter().any(|v| self.match_condition(v))
            }
            ConditionType::All => {
                let values: Vec<T> = group.value_as_vec();
                values.into_iter().all(|v| self.match_condition(v))
            }
            ConditionType::None => {
                let values: Vec<T> = group.value_as_vec();
                !values.into_iter().any(|v| self.match_condition(v))
            }
        }
    }
}
