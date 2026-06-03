use aho_corasick::{AhoCorasick, MatchKind};
use engine_detect_ner::{DetectionSource, EntitySpan};

const DEFAULT_KEYWORDS: &[&str] = &[
    "password",
    "api_key",
    "secret",
    "confidential",
    "internal only",
    "ssn",
    "aadhaar",
    "bank account",
];

pub fn detect_keywords(text: &str) -> Vec<EntitySpan> {
    detect_keywords_with_dictionary(text, DEFAULT_KEYWORDS)
}

pub fn detect_keywords_with_dictionary(text: &str, dictionary: &[&str]) -> Vec<EntitySpan> {
    if text.is_empty() || dictionary.is_empty() {
        return Vec::new();
    }

    let ac = match AhoCorasick::builder()
        .ascii_case_insensitive(true)
        .match_kind(MatchKind::LeftmostLongest)
        .build(dictionary)
    {
        Ok(x) => x,
        Err(_) => return Vec::new(),
    };

    let mut out = Vec::new();
    for m in ac.find_iter(text) {
        let matched = &text[m.start()..m.end()];
        out.push(EntitySpan {
            entity_type: "KEYWORD".to_string(),
            start: m.start(),
            end: m.end(),
            value: matched.to_string(),
            confidence: 1.0,
            source: DetectionSource::Keyword,
        });
    }
    out
}

