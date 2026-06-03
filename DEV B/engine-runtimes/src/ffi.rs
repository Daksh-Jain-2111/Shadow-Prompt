use engine_detect_ner::{merge_spans, DetectionSource, EntitySpan, MockNER, NERAdapter, UserRule};
use serde::Deserialize;

#[swift_bridge::bridge]
mod ffi {
    extern "Rust" {
        fn mask_text(input: String) -> String;
        fn mask_summary(input: String) -> String;
        fn unmask_reply(reply: String) -> String;
        fn unmask_reply_json(reply: String) -> String;
        fn wipe_stm();
        fn mask_with_rules_json(input: String, rules_json: String) -> String;
        fn mask_with_keywords_json(input: String, keywords_json: String) -> String;
    }
}

struct SummaryCounts {
    names: usize,
    pan: usize,
    aadhaar: usize,
}

fn build_counts(input: &str) -> SummaryCounts {
    let regex_spans = engine_detect_regex::scan_text(input)
        .into_iter()
        .filter(|d| d.is_valid && d.start < d.end && d.end <= input.len())
        .map(|d| engine_detect_ner::EntitySpan {
            entity_type: d.entity_type,
            start: d.start,
            end: d.end,
            value: d.value,
            confidence: 1.0,
            source: engine_detect_ner::DetectionSource::Regex,
        });

    let ner = MockNER::new();
    let ner_spans = ner
        .detect_entities(input)
        .into_iter()
        .filter(|s| s.start < s.end && s.end <= input.len());

    let merged = merge_spans(regex_spans.chain(ner_spans).collect());

    let mut names = 0usize;
    let mut pan = 0usize;
    let mut aadhaar = 0usize;

    for span in merged {
        match span.entity_type.as_str() {
            "PERSON" | "NAME" => names += 1,
            "PAN" => pan += 1,
            "AADHAAR" => aadhaar += 1,
            _ => {}
        }
    }

    SummaryCounts {
        names,
        pan,
        aadhaar,
    }
}

pub fn mask_text(input: String) -> String {
    crate::stm::mask_text_store(input, &[])
}

pub fn mask_summary(input: String) -> String {
    let counts = build_counts(&input);
    format!(
        "Hidden: {} name{}, {} PAN, {} Aadhaar",
        counts.names,
        if counts.names == 1 { "" } else { "s" },
        counts.pan,
        counts.aadhaar
    )
}

pub fn unmask_reply(reply: String) -> String {
    crate::stm::unmask_reply(reply)
}

pub fn unmask_reply_json(reply: String) -> String {
    crate::stm::unmask_reply_json(reply)
}

pub fn wipe_stm() {
    crate::stm::wipe_stm();
}

#[derive(Deserialize)]
struct RuleRow {
    pattern: String,
    replacement_type: String,
}

pub fn mask_with_rules_json(input: String, rules_json: String) -> String {
    let rows: Vec<RuleRow> = serde_json::from_str(&rules_json).unwrap_or_default();
    let rules: Vec<UserRule> = rows
        .into_iter()
        .map(|r| UserRule {
            pattern: r.pattern,
            replacement_type: r.replacement_type,
        })
        .collect();
    crate::stm::mask_text_store(input, &rules)
}

pub fn mask_with_keywords_json(input: String, keywords_json: String) -> String {
    let keys: Vec<String> = serde_json::from_str(&keywords_json).unwrap_or_default();
    let rules = keywords_to_rules(&keys);
    crate::stm::mask_text_store(input, &rules)
}

#[derive(Debug, Deserialize)]
struct CanonicalEntityRow {
    label: String,
    start: usize,
    end: usize,
    #[serde(default = "default_confidence")]
    confidence: f32,
}

fn default_confidence() -> f32 {
    1.0
}

fn keywords_to_rules(keys: &[String]) -> Vec<UserRule> {
    let mut rules = Vec::new();
    for k in keys {
        if k.is_empty() {
            continue;
        }
        let escaped = regex::escape(k.trim());
        rules.push(UserRule {
            pattern: format!(r"(?i)\b{escaped}\b"),
            replacement_type: "KEYWORD".into(),
        });
    }
    rules
}

fn parse_canonical_entities(input: &str, entities_json: &str) -> Vec<EntitySpan> {
    let rows: Vec<CanonicalEntityRow> = serde_json::from_str(entities_json).unwrap_or_default();
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        if row.start >= row.end || row.end > input.len() {
            continue;
        }
        let value = input[row.start..row.end].to_string();
        out.push(EntitySpan {
            entity_type: row.label,
            start: row.start,
            end: row.end,
            value,
            confidence: row.confidence,
            source: DetectionSource::NER,
        });
    }
    out
}

/// Mask with canonical entity JSON (UTF-8 byte offsets) plus optional keywords.
pub fn mask_with_ner_json(
    input: String,
    entities_json: String,
    keywords_json: String,
    use_mock_ner: bool,
) -> String {
    let extra = parse_canonical_entities(&input, &entities_json);
    let keys: Vec<String> = serde_json::from_str(&keywords_json).unwrap_or_default();
    let rules = keywords_to_rules(&keys);
    crate::stm::mask_text_store_with_ner(input, &rules, &extra, use_mock_ner)
}
