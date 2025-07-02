use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

pub use super::OneOrMany;
pub use super::file_evaluator::FileEvaluator;
pub use super::gatekeeper_evaluator::GatekeeperEvaluator;
pub use super::hostname_evaluator::HostnameEvaluator;
pub use super::os_evaluator::OSEvaluator;

macro_rules! evaluator_enum_and_impl {
    (
        $(#[$enum_meta:meta])*
        $vis:vis enum $EnumName:ident {
            $(
                $Variant:ident($Inner:ty)
            ),+ $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        $vis enum $EnumName {
            $(
                $Variant($Inner)
            ),+
        }

        impl $EnumName {
            pub fn match_eq(&self) -> Result<bool> {
                match self {
                    $(
                        $EnumName::$Variant(v) => v.match_eq(),
                    )+
                }
            }
            pub fn match_neq(&self) -> Result<bool> {
                match self {
                    $(
                        $EnumName::$Variant(v) => v.match_neq(),
                    )+
                }
            }
            pub fn match_any(&self) -> Result<bool> {
                match self {
                    $(
                        $EnumName::$Variant(v) => v.match_any(),
                    )+
                }
            }
            pub fn match_all(&self) -> Result<bool> {
                match self {
                    $(
                        $EnumName::$Variant(v) => v.match_all(),
                    )+
                }
            }
            pub fn match_none(&self) -> Result<bool> {
                match self {
                    $(
                        $EnumName::$Variant(v) => v.match_none(),
                    )+
                }
            }
        }
    }
}

evaluator_enum_and_impl! {
    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "lowercase", tag = "type", content = "args")]
    pub enum EvaluatorType {
        Hostname(OneOrMany<HostnameEvaluator>),
        File(OneOrMany<FileEvaluator>),
        Gatekeeper(OneOrMany<GatekeeperEvaluator>),
        Os(OneOrMany<OSEvaluator>),
    }
}
