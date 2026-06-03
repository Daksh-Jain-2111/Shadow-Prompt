use engine_detect_ner::{mask_pipeline_extended, EntitySpan, MockNER, UserRule};
use engine_mask_substitute::{unmask_exact_then_fuzzy, unmask_text, SessionMap};
use std::sync::{Mutex, OnceLock};

static STM: OnceLock<Mutex<Option<SessionMap>>> = OnceLock::new();

fn stm_lock() -> &'static Mutex<Option<SessionMap>> {
    STM.get_or_init(|| Mutex::new(None))
}

fn replace_session(new_session: SessionMap) {
    let mut g = stm_lock().lock().expect("stm poisoned");
    if let Some(mut old) = g.take() {
        old.secure_wipe();
    }
    *g = Some(new_session);
}

pub fn mask_text_store(input: String, rules: &[UserRule]) -> String {
    mask_text_store_with_ner(input, rules, &[], true)
}

/// Mask with optional ML spans (UTF-8 byte offsets). When `use_mock_ner` is true, MockNER still runs as fallback.
pub fn mask_text_store_with_ner(
    input: String,
    rules: &[UserRule],
    extra_ner: &[EntitySpan],
    use_mock_ner: bool,
) -> String {
    let mut session = SessionMap::new();
    let ner = MockNER::new();
    let out = mask_pipeline_extended(&input, &mut session, &ner, rules, extra_ner, use_mock_ner);
    log::debug!("STM: storing session in RAM only (no disk persistence)");
    replace_session(session);
    out
}

pub fn unmask_reply(reply: String) -> String {
    let g = stm_lock().lock().expect("stm poisoned");
    match &*g {
        Some(s) => unmask_text(&reply, s),
        None => reply,
    }
}

pub fn unmask_reply_json(reply: String) -> String {
    use serde::Serialize;
    #[derive(Serialize)]
    struct FuzzyDto {
        utf16_start: usize,
        utf16_end: usize,
        masked: String,
        revealed: String,
    }
    #[derive(Serialize)]
    struct OutDto {
        text: String,
        fuzzy_spans: Vec<FuzzyDto>,
    }

    let g = stm_lock().lock().expect("stm poisoned");
    match &*g {
        Some(s) => {
            let r = unmask_exact_then_fuzzy(&reply, s);
            let fuzzy_spans: Vec<FuzzyDto> = r
                .fuzzy_spans
                .into_iter()
                .map(|f| FuzzyDto {
                    utf16_start: f.utf16_start,
                    utf16_end: f.utf16_end,
                    masked: f.masked,
                    revealed: f.revealed,
                })
                .collect();
            serde_json::to_string(&OutDto {
                text: r.text,
                fuzzy_spans,
            })
            .unwrap_or_else(|_| reply.clone())
        }
        None => serde_json::to_string(&OutDto {
            text: reply,
            fuzzy_spans: vec![],
        })
        .unwrap_or_default(),
    }
}

pub fn wipe_stm() {
    let mut g = stm_lock().lock().expect("stm poisoned");
    if let Some(mut s) = g.take() {
        s.secure_wipe();
    }
    log::info!("STM zeroized successfully");
}
