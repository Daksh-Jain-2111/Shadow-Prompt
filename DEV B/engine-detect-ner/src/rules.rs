use regex::Regex;

use crate::entity::{DetectionSource, EntitySpan};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserRule {
    pub pattern: String,
    pub replacement_type: String,
}

pub fn apply_user_rules(text: &str, rules: &[UserRule]) -> Vec<EntitySpan> {
    let mut out = Vec::new();
    for rule in rules {
        let re = match Regex::new(&rule.pattern) {
            Ok(r) => r,
            Err(_) => continue,
        };
        for m in re.find_iter(text) {
            out.push(EntitySpan {
                entity_type: rule.replacement_type.clone(),
                start: m.start(),
                end: m.end(),
                value: m.as_str().to_string(),
                confidence: 1.0,
                source: DetectionSource::UserRule,
            });
        }
    }
    out
}

