use engine_runtime::{mask_text, unmask_reply, wipe_stm};

#[test]
fn mask_then_unmask_round_trip() {
    let masked = mask_text("Rahul PAN ABCDE1234F".into());
    assert!(!masked.contains("ABCDE1234F"));
    let restored = unmask_reply(masked.clone());
    assert!(restored.contains("ABCDE1234F"));
    wipe_stm();
    let no_session = unmask_reply(masked.clone());
    assert_eq!(no_session, masked);
}
