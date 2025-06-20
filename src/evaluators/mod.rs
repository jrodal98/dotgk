mod file_evaluator;
mod hostname_evaluator;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

// Define a trait for evaluators
trait EvaluatorTrait {
    fn evaluate(&self) -> bool;
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

#[derive(Serialize, Deserialize, Debug)]
struct HostnameEvaluator {
    target: String
}

#[derive(Serialize, Deserialize, Debug)]
struct FileEvaluator {
    path: String
}

impl EvaluatorTrait for FileEvaluator {
    fn evaluate(&self) -> bool {
        PathBuf::from(&self.path).exists()
    }
}

impl EvaluatorTrait for HostnameEvaluator {
    fn evaluate(&self) -> bool {
        self.target == hostname::get().unwrap().to_str().unwrap().to_string()
    }
}

enum OneOrManyRef<'a, T> {
    One(&'a T),
    Many(Vec<&'a T>),
}

impl<'a, T: EvaluatorTrait> OneOrManyRef<'a, T> {
    fn match_eq(&self) -> bool {
        self.iter().all(|v| v.evaluate())
    }

    fn match_neq(&self) -> bool {
        self.iter().all(|v| !v.evaluate())
    }

    fn match_any(&self) -> bool {
        self.iter().any(|v| v.evaluate())
    }

    fn match_all(&self) -> bool {
        self.iter().all(|v| v.evaluate())
    }

    fn match_none(&self) -> bool {
        self.iter().all(|v| !v.evaluate())
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        match self {
            OneOrManyRef::One(v) => Box::new(std::iter::once(*v)),
            OneOrManyRef::Many(v) => Box::new(v.iter().copied()),
        }
    }
}

impl<'a, T> From<&'a OneOrMany<T>> for OneOrManyRef<'a, T> {
    fn from(one_or_many: &'a OneOrMany<T>) -> Self {
        match one_or_many {
            OneOrMany::One(v) => OneOrManyRef::One(v),
            OneOrMany::Many(v) => OneOrManyRef::Many(v.iter().collect()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase", tag = "type", content = "args")]
pub enum EvaluatorType {
    Hostname(OneOrMany<HostnameEvaluator>),
    File(OneOrMany<FileEvaluator>),
}

impl Evaluator {
    pub fn evaluate(&self) -> bool {
        match &self.evaluator_type {
            EvaluatorType::File(v) => {
                let one_or_many = OneOrManyRef::from(v);
                match &self.condition {
                    ConditionType::Eq => one_or_many.match_eq(),
                    ConditionType::Neq => one_or_many.match_neq(),
                    ConditionType::Any => one_or_many.match_any(),
                    ConditionType::All => one_or_many.match_all(),
                    ConditionType::None => one_or_many.match_none(),
                }
            },
            EvaluatorType::Hostname(v) => {
                let one_or_many = OneOrManyRef::from(v);
                match &self.condition {
                    ConditionType::Eq => one_or_many.match_eq(),
                    ConditionType::Neq => one_or_many.match_neq(),
                    ConditionType::Any => one_or_many.match_any(),
                    ConditionType::All => one_or_many.match_all(),
                    ConditionType::None => one_or_many.match_none(),
                }
            },
        }
    }
}
