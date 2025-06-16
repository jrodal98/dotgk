use super::GroupEvaluator;


#[derive(Debug)]
pub struct HostnameEvaluator;

// Implement GroupEvaluator for HostnameEvaluator
impl GroupEvaluator<String> for HostnameEvaluator {
    fn match_condition(&self, value: String) -> bool {
        value == hostname::get().unwrap().to_str().unwrap().to_string()
    }
}
