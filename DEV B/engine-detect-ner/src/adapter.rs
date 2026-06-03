use regex::Regex;

use crate::entity::{DetectionSource, EntitySpan};

pub trait NERAdapter {
    fn detect_entities(&self, text: &str) -> Vec<EntitySpan>;
}

/// Mock NER adapter:
/// - PERSON: simple capitalized single-token names (e.g. "Rahul", "Arjun")
/// - ORG: simple "X Bank" patterns (e.g. "HDFC Bank", "ICICI Bank")
///
/// It is intentionally small and deterministic (no ML inference).
#[derive(Debug, Default)]
pub struct MockNER;

impl MockNER {
    pub fn new() -> Self {
        Self
    }
}

impl NERAdapter for MockNER {
    fn detect_entities(&self, text: &str) -> Vec<EntitySpan> {
        let mut out = Vec::new();

        // ORG: "<TOKEN> Bank" where token is 2-10 uppercase letters (HDFC, ICICI, SBI, etc.).
        let org_re = Regex::new(r"\b([A-Z]{2,10})\s+Bank\b").unwrap();
        for m in org_re.find_iter(text) {
            out.push(EntitySpan {
                entity_type: "ORG".to_string(),
                start: m.start(),
                end: m.end(),
                value: m.as_str().to_string(),
                confidence: 0.85,
                source: DetectionSource::NER,
            });
        }

        // PERSON: capitalized single token 2-20 letters. Avoid "Bank".
        let person_re = Regex::new(r"\b([A-Z][a-z]{1,19})\b").unwrap();
        for m in person_re.find_iter(text) {
            let v = m.as_str();
            if v == "Bank" {
                continue;
            }
            out.push(EntitySpan {
                entity_type: "PERSON".to_string(),
                start: m.start(),
                end: m.end(),
                value: v.to_string(),
                confidence: 0.70,
                source: DetectionSource::NER,
            });
        }

        out
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct EntitySpan {
    pub entity_type: String,
    pub start: usize,
    pub end: usize,
    pub value: String,
    pub confidence: f32,
    pub source: DetectionSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DetectionSource {
    Regex,
    NER,
    UserRule,
    Keyword,
}

impl EntitySpan {
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }
}


mod adapter;
mod entity;
mod merge;
mod pipeline;
mod rules;

pub use adapter::{MockNER, NERAdapter};
pub use entity::{DetectionSource, EntitySpan};
pub use merge::merge_spans;
pub use pipeline::{mask_pipeline, mask_pipeline_extended};
pub use rules::UserRule;


use std::collections::HashSet;

use crate::entity::{DetectionSource, EntitySpan};

fn priority(src: DetectionSource) -> u8 {
    match src {
        DetectionSource::UserRule => 3,
        DetectionSource::Regex => 2,
        DetectionSource::Keyword => 1,
        DetectionSource::NER => 0,
    }
}

fn better(a: &EntitySpan, b: &EntitySpan) -> bool {
    // true if a is a better choice than b
    let la = a.len();
    let lb = b.len();
    if la != lb {
        return la > lb;
    }
    let pa = priority(a.source);
    let pb = priority(b.source);
    if pa != pb {
        return pa > pb;
    }
    if a.start != b.start {
        return a.start < b.start;
    }
    if a.end != b.end {
        return a.end > b.end;
    }
    a.entity_type < b.entity_type
}

fn overlaps(a: &EntitySpan, b: &EntitySpan) -> bool {
    a.start < b.end && b.start < a.end
}

/// Merge spans using:
/// - longest match wins
/// - priority: UserRule > Regex > NER
/// - remove overlapping duplicates
///
/// The implementation treats any chain of overlaps as one group and selects the single best span.
pub fn merge_spans(mut spans: Vec<EntitySpan>) -> Vec<EntitySpan> {
    spans.retain(|s| s.start < s.end);
    if spans.is_empty() {
        return spans;
    }

    // Remove exact duplicates first (including source + entity_type).
    let mut seen: HashSet<(usize, usize, String, DetectionSource)> = HashSet::new();
    spans.retain(|s| {
        seen.insert((s.start, s.end, s.entity_type.clone(), s.source))
    });

    spans.sort_by(|a, b| {
        a.start
            .cmp(&b.start)
            .then_with(|| b.end.cmp(&a.end)) // longer first when same start
            .then_with(|| priority(b.source).cmp(&priority(a.source))) // higher priority first
            .then_with(|| a.entity_type.cmp(&b.entity_type))
    });

    let mut out: Vec<EntitySpan> = Vec::new();
    let mut cur_best: Option<EntitySpan> = None;
    let mut group_end: usize = 0;

    for s in spans.into_iter() {
        match cur_best.as_ref() {
            None => {
                group_end = s.end;
                cur_best = Some(s);
            }
            Some(best) => {
                if s.start < group_end {
                    if s.end > group_end {
                        group_end = s.end;
                    }
                    if better(&s, best) {
                        cur_best = Some(s);
                    }
                } else {
                    out.push(cur_best.take().unwrap());
                    group_end = s.end;
                    cur_best = Some(s);
                }
            }
        }
    }

    if let Some(last) = cur_best {
        out.push(last);
    }

    // Final safety: ensure non-overlap and stable ordering.
    out.sort_by_key(|s| s.start);
    let mut non_overlap: Vec<EntitySpan> = Vec::with_capacity(out.len());
    for s in out.into_iter() {
        if non_overlap.last().is_some_and(|p| overlaps(p, &s)) {
            // Should not happen, but if it does: keep the better one.
            let mut prev = non_overlap.pop().unwrap();
            if better(&s, &prev) {
                prev = s;
            }
            non_overlap.push(prev);
        } else {
            non_overlap.push(s);
        }
    }
    non_overlap
}


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

