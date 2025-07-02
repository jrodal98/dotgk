use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use std::ffi::OsString;
use super::EvaluatorTrait;

#[derive(Serialize, Deserialize, Debug)]
pub struct HostnameEvaluator {
    target: String,
}

// this is kind of stupid, but I don't want to deal with mocking right now
impl HostnameEvaluator {
    #[cfg(not(test))]
    fn get_hostname() -> Result<OsString> {
        hostname::get().context("Failed to get hostname")
    }

    #[cfg(test)]
    fn get_hostname() -> Result<OsString> {
        Ok("test-hostname".into())    
    }
}

impl EvaluatorTrait for HostnameEvaluator {
    fn evaluate(&self) -> Result<bool> {
        let hostname = Self::get_hostname()?;
        let hostname_str = hostname
            .to_str()
            .context("Failed to convert hostname to string")?;
        Ok(self.target == hostname_str)
    }
}


#[cfg(test)]
mod tests {
    use crate::gatekeeper::test_helper;
    use anyhow::Result;

    #[test]
    fn test_pass() -> Result<()> {
        test_helper("hostname_pass", true)
    }

    #[test]
    fn test_fail() -> Result<()> {
        test_helper("hostname_fail", false)
    }
}
