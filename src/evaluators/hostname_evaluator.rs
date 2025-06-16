use super::GroupEvaluator;
use anyhow::Context;


#[derive(Debug)]
pub struct HostnameEvaluator;

// Implement GroupEvaluator for HostnameEvaluator
impl GroupEvaluator<String> for HostnameEvaluator {
    fn single_passes(&self, value: String) -> bool {
        value == hostname::get().unwrap().to_str().unwrap().to_string()
    }
}
