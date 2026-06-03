use engine_mask_substitute::SessionMap;

use crate::adapter::NERAdapter;
use crate::entity::{DetectionSource, EntitySpan};
use crate::merge::merge_spans;
use crate::rules::{apply_user_rules, UserRule};

fn regex_detect(text: &str) -> Vec<EntitySpan> {
    engine_detect_regex::scan_text(text)
        .into_iter()
        .filter(|d| d.is_valid && d.start < d.end && d.end <= text.len())
        .map(|d| EntitySpan {
            entity_type: d.entity_type,
            start: d.start,
            end: d.end,
            value: d.value,
            confidence: 1.0,
            source: DetectionSource::Regex,
        })
        .collect()
}

fn normalize_for_substitution(entity_type: &str) -> &str {
    match entity_type {
        "PERSON" => "NAME",
        other => other,
    }
}

fn apply_substitutions_back_to_front(
    text: &str,
    session: &mut SessionMap,
    spans: &[EntitySpan],
) -> String {
    if spans.is_empty() {
        return text.to_string();
    }

    let mut out = text.to_string();
    let mut ordered: Vec<&EntitySpan> = spans.iter().collect();
    ordered.sort_by(|a, b| b.start.cmp(&a.start).then_with(|| b.end.cmp(&a.end)));

    for s in ordered {
        if s.start >= s.end || s.end > out.len() {
            continue;
        }
        let original = &out[s.start..s.end];
        let et = normalize_for_substitution(&s.entity_type);
        let sub = session.get_or_create(et, original);
        out.replace_range(s.start..s.end, &sub);
    }

    out
}

fn valid_span(s: &EntitySpan, text_len: usize) -> bool {
    s.start < s.end && s.end <= text_len
}

/// Hybrid masking pipeline with optional externally supplied NER spans (e.g. TFLite on Android).
pub fn mask_pipeline_extended(
    text: &str,
    session: &mut SessionMap,
    ner: &dyn NERAdapter,
    rules: &[UserRule],
    extra_ner: &[EntitySpan],
    use_mock_ner: bool,
) -> String {
    let text_len = text.len();
    let mut spans = Vec::new();

    spans.extend(regex_detect(text));

    spans.extend(
        extra_ner
            .iter()
            .filter(|s| valid_span(s, text_len))
            .cloned(),
    );

    if use_mock_ner {
        spans.extend(
            ner.detect_entities(text)
                .into_iter()
                .filter(|s| valid_span(s, text_len)),
        );
    }

    spans.extend(apply_user_rules(text, rules));

    let merged = merge_spans(spans);
    apply_substitutions_back_to_front(text, session, &merged)
}

/// Hybrid masking pipeline:
/// text input → regex detection + NER detection + user rules → merged spans →
/// SessionMap substitution → masked output
pub fn mask_pipeline(
    text: &str,
    session: &mut SessionMap,
    ner: &dyn NERAdapter,
    rules: &[UserRule],
) -> String {
    mask_pipeline_extended(text, session, ner, rules, &[], true)
}

