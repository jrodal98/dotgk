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
    pub value: Value,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum EvaluatorType {
    Hostname,
    File,
}

pub trait GroupEvaluator<T> 
where
    T: for<'de> serde::Deserialize<'de> + PartialEq,
{
    fn evaluate(&self, group: &Evaluator) -> bool 
    {
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
    fn match_condition(&self, value: T) -> bool;
}

impl Evaluator {
    pub fn evaluate(&self) -> bool {
        match self.evaluator_type {
            EvaluatorType::Hostname => crate::evaluators::hostname_evaluator::HostnameEvaluator.evaluate(self),
            EvaluatorType::File => crate::evaluators::file_evaluator::FileEvaluator.evaluate(self),
        }

    }

    fn value_as_vec<T>(&self) -> Vec<T> 
    where
        T: for<'de> serde::Deserialize<'de> + PartialEq,
    {
        self.value.as_array().unwrap().into_iter().map(|v| serde_json::from_value(v.clone()).unwrap()).collect()
    }

    fn value_as_single<T>(&self) -> T
    where
        T: for<'de> serde::Deserialize<'de> + PartialEq,
    {
        serde_json::from_value(self.value.clone()).unwrap()
    }
}
