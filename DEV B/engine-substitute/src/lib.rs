//! Lightweight redaction helpers (separate from the main mask engine).

use regex::Regex;
use std::sync::OnceLock;

struct Patterns {
    email: Regex,
    aadhaar: Regex,
    pan: Regex,
    phone: Regex,
}

fn patterns() -> &'static Patterns {
    static P: OnceLock<Patterns> = OnceLock::new();
    P.get_or_init(|| Patterns {
        email: Regex::new(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}").expect("email re"),
        aadhaar: Regex::new(r"\b\d{12}\b").expect("aadhaar re"),
        pan: Regex::new(r"\b[A-Z]{5}[0-9]{4}[A-Z]\b").expect("pan re"),
        phone: Regex::new(r"\b[6-9]\d{9}\b").expect("phone re"),
    })
}

fn mask_run(mask_char: char, len: usize) -> String {
    std::iter::repeat(mask_char).take(len).collect()
}

/// Best-effort PII redaction using fixed patterns (Indian phone, PAN, 12-digit ID, email).
pub fn redact_pii(input: &str, mask_char: char) -> String {
    let p = patterns();
    let mut out = input.to_string();
    out = p
        .email
        .replace_all(&out, |m: &regex::Captures<'_>| {
            let full = m.get(0).unwrap();
            mask_run(mask_char, full.end() - full.start())
        })
        .into_owned();
    out = p
        .aadhaar
        .replace_all(&out, |m: &regex::Captures<'_>| {
            let full = m.get(0).unwrap();
            mask_run(mask_char, full.end() - full.start())
        })
        .into_owned();
    out = p
        .pan
        .replace_all(&out, |m: &regex::Captures<'_>| {
            let full = m.get(0).unwrap();
            mask_run(mask_char, full.end() - full.start())
        })
        .into_owned();
    out = p
        .phone
        .replace_all(&out, |m: &regex::Captures<'_>| {
            let full = m.get(0).unwrap();
            mask_run(mask_char, full.end() - full.start())
        })
        .into_owned();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_pan_phone_email() {
        let s = redact_pii("PAN ABCDE1234F call 9876543210 mail a@b.co", 'X');
        assert!(!s.contains("ABCDE1234F"));
        assert!(!s.contains("9876543210"));
        assert!(!s.contains("a@b.co"));
        assert!(s.contains("XXXXXXXXXX"));
    }
}
