use engine_detect_ner::{mask_pipeline, DetectionSource, EntitySpan, NERAdapter, UserRule};
use engine_mask_substitute::SessionMap;

struct TestNER {
    spans: Vec<EntitySpan>,
}

impl NERAdapter for TestNER {
    fn detect_entities(&self, _text: &str) -> Vec<EntitySpan> {
        self.spans.clone()
    }
}

#[test]
fn regex_beats_ner_on_equal_overlap() {
    let text = "ABCDE1234F";
    let mut session = SessionMap::new();

    let ner = TestNER {
        spans: vec![EntitySpan {
            entity_type: "ORG".to_string(),
            start: 0,
            end: 10,
            value: "ABCDE1234F".to_string(),
            confidence: 0.9,
            source: DetectionSource::NER,
        }],
    };

    let masked = mask_pipeline(text, &mut session, &ner, &[]);
    assert_ne!(masked, text);

    let before = session.forward.len();
    let _pan_sub = session.get_or_create("PAN", "ABCDE1234F");
    assert_eq!(
        session.forward.len(),
        before,
        "PAN mapping should already exist if Regex won overlap"
    );
}

#[test]
fn user_rule_overrides_regex_and_ner() {
    let text = "ABCDE1234F";
    let mut session = SessionMap::new();

    let ner = TestNER { spans: vec![] };

    let rules = vec![UserRule {
        pattern: r"\b[A-Z]{5}[0-9]{4}[A-Z]\b".to_string(),
        replacement_type: "NAME".to_string(),
    }];

    let _masked = mask_pipeline(text, &mut session, &ner, &rules);
    let before = session.forward.len();
    let _name_sub = session.get_or_create("NAME", "ABCDE1234F");
    assert_eq!(
        session.forward.len(),
        before,
        "NAME mapping should already exist if UserRule won overlap"
    );
}

#[test]
fn session_consistency_same_input_same_output() {
    let text = "Rahul PAN ABCDE1234F works at HDFC Bank";
    let mut session = SessionMap::new();

    let ner = engine_detect_ner::MockNER::new();
    let rules: Vec<UserRule> = vec![];

    let masked1 = mask_pipeline(text, &mut session, &ner, &rules);
    let masked2 = mask_pipeline(text, &mut session, &ner, &rules);
    assert_eq!(masked1, masked2);
}

#[test]
fn mixed_entity_masking_creates_session_mappings() {
    let text = "Rahul PAN ABCDE1234F works at HDFC Bank and email is rahul@example.com";
    let mut session = SessionMap::new();
    let ner = engine_detect_ner::MockNER::new();

    let masked = mask_pipeline(text, &mut session, &ner, &[]);
    assert!(!masked.contains("ABCDE1234F"));
    assert!(!masked.contains("rahul@example.com"));

    // Ensure mappings used by pipeline already exist.
    let before = session.forward.len();
    let pan_sub = session.get_or_create("PAN", "ABCDE1234F");
    assert_eq!(session.forward.len(), before);
    assert!(masked.contains(&pan_sub));

    let before = session.forward.len();
    let email_sub = session.get_or_create("EMAIL", "rahul@example.com");
    assert_eq!(session.forward.len(), before);
    assert!(masked.contains(&email_sub));
}

#[test]
fn overlap_safety_longest_match_wins() {
    let text = "ABCDE1234F";
    let mut session = SessionMap::new();
    let ner = TestNER { spans: vec![] };

    let rules = vec![
        UserRule {
            pattern: r"\bABCDE1234F\b".to_string(),
            replacement_type: "PAN".to_string(),
        },
        UserRule {
            pattern: r"\bBCDE123\b".to_string(),
            replacement_type: "PAN".to_string(),
        },
    ];

    let masked = mask_pipeline(text, &mut session, &ner, &rules);
    assert_ne!(masked, text);

    let before = session.forward.len();
    let _pan_sub = session.get_or_create("PAN", "ABCDE1234F");
    assert_eq!(
        session.forward.len(),
        before,
        "Longest full-span rule should have been applied"
    );
}

