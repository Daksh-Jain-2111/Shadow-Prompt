use engine_mask_substitute::{generate_substitute, mask_text, unmask_text, Detection, SessionMap};

fn det(entity_type: &str, start: usize, end: usize, value: &str, valid: bool) -> Detection {
    Detection {
        entity_type: entity_type.to_string(),
        start,
        end,
        value: value.to_string(),
        is_valid: valid,
    }
}

#[test]
fn consistent_substitution_same_session() {
    let mut session = SessionMap::new();
    let a = session.get_or_create("NAME", "Rahul");
    let b = session.get_or_create("NAME", "Rahul");
    assert_eq!(a, b);
    assert_ne!(a, "Rahul");
}

#[test]
fn masking_removes_plaintext_pan() {
    let input = "My PAN is ABCDE1234F";
    let dets = vec![det("PAN", 10, 20, "ABCDE1234F", true)];
    let mut session = SessionMap::new();
    let masked = mask_text(input, &dets, &mut session);
    assert!(!masked.contains("ABCDE1234F"));
    assert!(masked.len() >= input.len().saturating_sub(5));
}

#[test]
fn unmask_restores_from_model_reply() {
    let mut session = SessionMap::new();
    session.insert_mapping("PAN", "ABCDE1234F", "ZXCVB5678Q");
    session.insert_mapping("NAME", "Rahul", "Arjun");

    let model = "Arjun should verify ZXCVB5678Q";
    let restored = unmask_text(model, &session);
    assert_eq!(restored, "Rahul should verify ABCDE1234F");
}

#[test]
fn multi_entity_masking() {
    let input = "PAN ABCDE1234F email a@b.com phone 9876543210";
    let dets = vec![
        det("PAN", 4, 14, "ABCDE1234F", true),
        det("EMAIL", 21, 26, "a@b.com", true),
        det("PHONE", 34, 44, "9876543210", true),
    ];
    let mut session = SessionMap::new();
    let masked = mask_text(input, &dets, &mut session);
    assert!(!masked.contains("ABCDE1234F"));
    assert!(!masked.contains("a@b.com"));
    assert!(!masked.contains("9876543210"));

    let back = unmask_text(&masked, &session);
    assert!(back.contains("ABCDE1234F"));
    assert!(back.contains("a@b.com"));
    assert!(back.contains("9876543210"));
}

#[test]
fn overlapping_detections_do_not_corrupt() {
    // Outer span fully contains inner; inner is dropped so the full PAN is replaced once.
    let input = "xxABCDE1234Fyy";
    let dets = vec![
        det("PAN", 2, 12, "ABCDE1234F", true),
        det("PAN", 4, 10, "CDE123", true),
    ];
    let mut session = SessionMap::new();
    let masked = mask_text(input, &dets, &mut session);
    assert!(!masked.contains("ABCDE1234F"));
    assert!(!masked.contains("CDE123"));
}

#[test]
fn session_consistency_across_mask_calls() {
    let input = "ABCDE1234F";
    let dets = vec![det("PAN", 0, 10, "ABCDE1234F", true)];
    let mut session = SessionMap::new();
    let m1 = mask_text(input, &dets, &mut session);
    let m2 = mask_text(input, &dets, &mut session);
    assert_eq!(m1, m2);
}

#[test]
fn invalid_detection_skipped_no_leak_policy() {
    let input = "ABCDE1234F";
    let dets = vec![det("PAN", 0, 10, "ABCDE1234F", false)];
    let mut session = SessionMap::new();
    let masked = mask_text(input, &dets, &mut session);
    assert_eq!(masked, input);
}

#[test]
fn unmask_longest_substitute_first() {
    let mut session = SessionMap::new();
    session.insert_mapping("A", "first", "ZZZ");
    session.insert_mapping("B", "second", "ZZZA");
    let out = unmask_text("x ZZZA y ZZZ z", &session);
    assert_eq!(out, "x second y first z");
}

#[test]
fn generate_substitute_shapes() {
    let pan = generate_substitute("PAN", "ABCDE1234F");
    assert_eq!(pan.len(), 10);
    assert!(pan[..5].chars().all(|c| c.is_ascii_uppercase()));
    assert!(pan[5..9].chars().all(|c| c.is_ascii_digit()));
    assert!(pan[9..].chars().all(|c| c.is_ascii_uppercase()));

    let phone = generate_substitute("PHONE", "9876543210");
    assert_eq!(phone.len(), 10);
    assert!(matches!(phone.chars().next().unwrap(), '6'..='9'));

    let email = generate_substitute("EMAIL", "rahul@gmail.com");
    assert!(email.contains('@'));
    assert!(email.ends_with(".masked.dev") || email.contains("masked.dev"));
}

#[test]
fn scan_then_mask_no_plaintext() {
    use engine_detect_regex::scan_text;

    fn convert(d: engine_detect_regex::Detection) -> Detection {
        Detection {
            entity_type: d.entity_type,
            start: d.start,
            end: d.end,
            value: d.value,
            is_valid: d.is_valid,
        }
    }

    let input = "Contact 9876543210 and ABCDE1234F";
    let dets: Vec<_> = scan_text(input).into_iter().map(convert).collect();
    let mut session = SessionMap::new();
    let masked = mask_text(input, &dets, &mut session);
    assert!(!masked.contains("9876543210"));
    assert!(!masked.contains("ABCDE1234F"));
}
