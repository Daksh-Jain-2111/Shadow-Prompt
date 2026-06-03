use engine_detect_ner::{mask_pipeline_extended, DetectionSource, EntitySpan, MockNER, UserRule};
use engine_mask_substitute::SessionMap;

#[test]
fn extra_ner_span_is_masked_without_mock() {
    let text = "Contact me at secret@corp.com";
    let mut session = SessionMap::new();
    let ner = MockNER::new();
    let extra = vec![EntitySpan {
        entity_type: "EMAIL".to_string(),
        start: text.find("secret@corp.com").unwrap(),
        end: text.find("secret@corp.com").unwrap() + "secret@corp.com".len(),
        value: "secret@corp.com".to_string(),
        confidence: 0.99,
        source: DetectionSource::NER,
    }];
    let masked = mask_pipeline_extended(text, &mut session, &ner, &[], &extra, false);
    assert!(!masked.contains("secret@corp.com"));
}

#[test]
fn mock_ner_still_runs_when_enabled() {
    let text = "Rahul PAN ABCDE1234F";
    let mut session = SessionMap::new();
    let ner = MockNER::new();
    let masked = mask_pipeline_extended(text, &mut session, &ner, &[], &[], true);
    assert!(!masked.contains("ABCDE1234F"));
}
