use engine_runtime::{mask_with_ner_json, unmask_reply, wipe_stm};

#[test]
fn mask_with_ner_json_round_trip() {
    wipe_stm();
    let input = "Email secret@corp.com today";
    let start = input.find("secret@corp.com").unwrap();
    let end = start + "secret@corp.com".len();
    let entities = format!(
        r#"[{{"label":"EMAIL","start":{start},"end":{end},"confidence":0.99}}]"#
    );
    let masked = mask_with_ner_json(input.to_string(), entities, "[]".to_string(), false);
    assert!(!masked.contains("secret@corp.com"));
    let restored = unmask_reply(masked.clone());
    assert!(restored.contains("secret@corp.com"));
    wipe_stm();
}
