use engine_detect_ner::{merge_spans, DetectionSource, EntitySpan};

use crate::chunking::{chunk_text, token_spans};
use crate::dictionary::detect_keywords;

const MAX_TOKENS: usize = 50_000;
const CHUNK_SIZE: usize = 512;
const CHUNK_OVERLAP: usize = 64;

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

pub fn tier4_detect(text: &str) -> Result<Vec<EntitySpan>, String> {
    let token_spans = token_spans(text);
    if token_spans.len() > MAX_TOKENS {
        return Err("Input exceeds maximum token limit".to_string());
    }

    let chunks = chunk_text(text, CHUNK_SIZE, CHUNK_OVERLAP);
    if chunks.is_empty() {
        return Ok(Vec::new());
    }

    let mut all_spans = Vec::new();
    for chunk in chunks {
        let chunk_byte_start = token_spans[chunk.start_token].start;

        for mut s in regex_detect(&chunk.text) {
            s.start += chunk_byte_start;
            s.end += chunk_byte_start;
            all_spans.push(s);
        }

        for mut s in detect_keywords(&chunk.text) {
            s.start += chunk_byte_start;
            s.end += chunk_byte_start;
            all_spans.push(s);
        }
    }

    Ok(merge_spans(all_spans))
}

