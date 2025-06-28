mod file_evaluator;
mod hostname_evaluator;

pub use file_evaluator::FileEvaluator;
pub use hostname_evaluator::HostnameEvaluator;
use serde::Deserialize;
use serde::Serialize;

// Define a trait for evaluators
pub trait EvaluatorTrait {
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase", tag = "type", content = "args")]
pub enum EvaluatorType {
    Hostname(OneOrMany<HostnameEvaluator>),
    File(OneOrMany<FileEvaluator>),
}

impl EvaluatorType {
    fn match_eq(&self) -> bool {
        match self {
            EvaluatorType::File(v) => v.match_eq(),
            EvaluatorType::Hostname(v) => v.match_eq(),
        }
    }

    fn match_neq(&self) -> bool {
        match self {
            EvaluatorType::File(v) => v.match_neq(),
            EvaluatorType::Hostname(v) => v.match_neq(),
        }
    }

    fn match_any(&self) -> bool {
        match self {
            EvaluatorType::File(v) => v.match_any(),
            EvaluatorType::Hostname(v) => v.match_any(),
        }
    }

    fn match_all(&self) -> bool {
        match self {
            EvaluatorType::File(v) => v.match_all(),
            EvaluatorType::Hostname(v) => v.match_all(),
        }
    }

    fn match_none(&self) -> bool {
        match self {
            EvaluatorType::File(v) => v.match_none(),
            EvaluatorType::Hostname(v) => v.match_none(),
        }
    }
}

impl Evaluator {
    pub fn evaluate(&self) -> bool {
        match &self.condition {
            ConditionType::Eq => self.evaluator_type.match_eq(),
            ConditionType::Neq => self.evaluator_type.match_neq(),
            ConditionType::Any => self.evaluator_type.match_any(),
            ConditionType::All => self.evaluator_type.match_all(),
            ConditionType::None => self.evaluator_type.match_none(),
        }
    }
}
