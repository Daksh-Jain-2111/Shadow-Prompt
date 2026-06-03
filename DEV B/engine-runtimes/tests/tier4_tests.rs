use std::collections::HashSet;
use std::time::Instant;

use engine_detect_ner::DetectionSource;
use engine_runtime::{chunk_text, detect_keywords, tier4_detect};

#[test]
fn chunk_generation_basic() {
    let text = (0..1000)
        .map(|i| format!("w{i}"))
        .collect::<Vec<_>>()
        .join(" ");
    let chunks = chunk_text(&text, 512, 64);
    assert!(!chunks.is_empty());
    assert_eq!(chunks[0].start_token, 0);
    assert!(chunks[0].end_token <= 512);
    for c in &chunks {
        assert!(c.start_token < c.end_token);
        assert!(!c.text.is_empty());
    }
}

#[test]
fn chunk_overlap_correctness() {
    let text = (0..1200)
        .map(|i| format!("tok{i}"))
        .collect::<Vec<_>>()
        .join(" ");
    let chunks = chunk_text(&text, 512, 64);
    assert!(chunks.len() >= 3);
    for pair in chunks.windows(2) {
        let a = &pair[0];
        let b = &pair[1];
        assert!(b.start_token < a.end_token);
        assert_eq!(a.end_token - b.start_token, 64);
    }
}

#[test]
fn token_cap_rejection() {
    let text = (0..50_001)
        .map(|i| format!("t{i}"))
        .collect::<Vec<_>>()
        .join(" ");
    let err = tier4_detect(&text).unwrap_err();
    assert_eq!(err, "Input exceeds maximum token limit");
}

#[test]
fn keyword_detection_case_insensitive() {
    let text = "This is CONFIDENTIAL and internal only with Api_Key value";
    let spans = detect_keywords(text);
    let values: Vec<String> = spans.into_iter().map(|s| s.value.to_lowercase()).collect();
    assert!(values.iter().any(|v| v == "confidential"));
    assert!(values.iter().any(|v| v == "internal only"));
    assert!(values.iter().any(|v| v == "api_key"));
}

#[test]
fn regex_and_keyword_merge_and_dedup() {
    let text = "contact me at test@example.com this is secret secret";
    let spans = tier4_detect(text).unwrap();
    assert!(!spans.is_empty());

    let mut has_regex = false;
    let mut has_keyword = false;
    let mut seen = HashSet::new();
    for s in spans {
        if s.source == DetectionSource::Regex && s.entity_type == "EMAIL" {
            has_regex = true;
        }
        if s.source == DetectionSource::Keyword {
            has_keyword = true;
        }
        assert!(seen.insert((s.start, s.end, s.entity_type.clone())));
    }

    assert!(has_regex);
    assert!(has_keyword);
}

#[test]
fn synthetic_5000_word_input_stability() {
    let mut words = Vec::with_capacity(5000);
    for i in 0..5000 {
        if i % 700 == 0 {
            words.push("john.doe@example.com".to_string());
        } else if i % 900 == 0 {
            words.push("secret".to_string());
        } else if i % 1100 == 0 {
            words.push("123-45-6789".to_string());
        } else if i % 1300 == 0 {
            words.push("aadhaar".to_string());
        } else {
            words.push(format!("word{i}"));
        }
    }
    let text = words.join(" ");
    let spans = tier4_detect(&text).unwrap();

    let mut seen = HashSet::new();
    for s in &spans {
        assert!(s.start < s.end);
        assert!(s.end <= text.len());
        assert!(seen.insert((s.start, s.end, s.entity_type.clone())));
    }
    assert!(!spans.is_empty());
}

#[test]
fn large_input_performance_sanity() {
    let mut words = Vec::with_capacity(5000);
    for i in 0..5000 {
        if i % 333 == 0 {
            words.push("confidential".to_string());
        } else {
            words.push(format!("x{i}"));
        }
    }
    let text = words.join(" ");
    let now = Instant::now();
    let spans = tier4_detect(&text).unwrap();
    let elapsed = now.elapsed();
    assert!(!spans.is_empty());
    assert!(
        elapsed.as_secs() < 5,
        "tier4 detection too slow: {:?}",
        elapsed
    );
}

