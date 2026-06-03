//! Deterministic-looking substitutes. Session-scoped consistency is enforced by `SessionMap`.

use rand::distributions::{Alphanumeric, DistString};
use rand::Rng;
use regex::Regex;
use std::borrow::Cow;

const INDIAN_FIRST_NAMES: &[&str] = &[
    "Arjun", "Rohan", "Vikram", "Aditya", "Karan", "Rahul", "Siddharth", "Aryan", "Dev", "Kabir",
    "Ananya", "Priya", "Kavya", "Isha", "Neha", "Meera", "Diya", "Anika", "Riya", "Sneha",
];

const SAFE_URL_DOMAINS: &[&str] = &[
    "https://safe.example.org",
    "https://placeholder.masked.dev",
    "https://redacted.invalid",
];

/// Normalize detector labels to generator families.
pub(crate) fn normalize_entity_type(entity_type: &str) -> Cow<'_, str> {
    match entity_type {
        "INDIAN_PHONE" => Cow::Borrowed("PHONE"),
        "IPV4" | "IPV6" => Cow::Borrowed("IP"),
        "DATES" => Cow::Borrowed("DATE"),
        other => Cow::Borrowed(other),
    }
}

/// Generate a fresh substitute for `original` (call only when no session mapping exists).
pub fn generate_substitute(entity_type: &str, original: &str) -> String {
    let norm = normalize_entity_type(entity_type);
    match norm.as_ref() {
        "PAN" => gen_pan(),
        "AADHAAR" => gen_aadhaar(),
        "PHONE" => gen_indian_mobile(),
        "EMAIL" => gen_email(original),
        "CREDIT_CARD" => gen_card_like(original),
        "NAME" => gen_name(),
        "MONEY" => gen_money(original),
        "DATE" => gen_date_shift(original, 0),
        "IP" => {
            if original.contains(':') {
                gen_ipv6()
            } else {
                gen_ipv4()
            }
        }
        "URL" => gen_url_placeholder(),
        _ => gen_generic_token(original),
    }
}

/// Apply the session-wide calendar shift for date-like entities.
pub(crate) fn generate_substitute_date(original: &str, offset_days: i32) -> String {
    gen_date_shift(original, offset_days)
}

fn gen_pan() -> String {
    let mut rng = rand::thread_rng();
    let letters: String = (0..5)
        .map(|_| rng.gen_range(b'A'..=b'Z') as char)
        .collect();
    let digits: String = (0..4).map(|_| rng.gen_range(b'0'..=b'9') as char).collect();
    let last = rng.gen_range(b'A'..=b'Z') as char;
    format!("{letters}{digits}{last}")
}

fn gen_aadhaar() -> String {
    let mut rng = rand::thread_rng();
    (0..12)
        .map(|_| rng.gen_range(b'0'..=b'9') as char)
        .collect()
}

fn gen_indian_mobile() -> String {
    let mut rng = rand::thread_rng();
    let first = rng.gen_range(6..=9);
    let rest: String = (0..9)
        .map(|_| rng.gen_range(b'0'..=b'9') as char)
        .collect();
    format!("{first}{rest}")
}

fn gen_email(original: &str) -> String {
    let mut rng = rand::thread_rng();
    let digits: String = (0..3).map(|_| rng.gen_range(b'0'..=b'9') as char).collect();
    let local = format!("user{digits}{}", rng.gen_range(10..99));
    if let Some(at) = original.find('@') {
        let domain = original[at + 1..].trim().to_lowercase();
        if domain.contains("gmail.com") || domain.contains("yahoo.") || domain.contains("outlook.") {
            return format!("{local}@masked.dev");
        }
        if !domain.is_empty() && domain.contains('.') {
            // Preserve registrable shape with a safe synthetic TLD.
            let synthetic = domain
                .rsplit_once('.')
                .map(|(base, _tld)| format!("{base}.masked.dev"))
                .unwrap_or_else(|| "masked.dev".to_string());
            return format!("{local}@{synthetic}");
        }
    }
    format!("{local}@masked.dev")
}

fn gen_card_like(original: &str) -> String {
    let digits: String = original.chars().filter(|c| c.is_ascii_digit()).collect();
    let len = digits.len().clamp(13, 19);
    let mut rng = rand::thread_rng();
    let body: String = (0..len.saturating_sub(1))
        .map(|_| rng.gen_range(b'0'..=b'9') as char)
        .collect();
    let last = rng.gen_range(0..10);
    format!("{body}{last}")
}

fn gen_name() -> String {
    let mut rng = rand::thread_rng();
    INDIAN_FIRST_NAMES[rng.gen_range(0..INDIAN_FIRST_NAMES.len())].to_string()
}

fn gen_money(original: &str) -> String {
    let mut rng = rand::thread_rng();
    let re = Regex::new(r"[-+]?\d*\.?\d+").unwrap();
    if let Some(m) = re.find(original) {
        if let Ok(v) = m.as_str().parse::<f64>() {
            let factor = rng.gen_range(1.10..=1.25);
            let inv = rng.gen_range(0.75..=0.90);
            let use_up = rng.gen_bool(0.5);
            let nv = if use_up { v * factor } else { v * inv };
            let prefix = original[..m.start()].to_string();
            let suffix = original[m.end()..].to_string();
            return format!("{prefix}{:.2}{suffix}", nv);
        }
    }
    let jitter = rng.gen_range(0.80..=1.20);
    format!("{:.2}", jitter * 1000.0)
}

/// Shift ISO `YYYY-MM-DD` or `DD/MM/YYYY` / `DD-MM-YYYY` by `offset_days` (negative allowed).
fn gen_date_shift(original: &str, offset_days: i32) -> String {
    let iso = Regex::new(r"\b(\d{4})-(\d{2})-(\d{2})\b").unwrap();
    let dmy = Regex::new(r"\b(\d{2})[/-](\d{2})[/-](\d{4})\b").unwrap();

    if let Some(c) = iso.captures(original) {
        let y: i32 = c[1].parse().unwrap_or(1970);
        let m: u32 = c[2].parse().unwrap_or(1);
        let d: u32 = c[3].parse().unwrap_or(1);
        if let Some(shifted) = shift_ymd(y, m, d, offset_days) {
            return original.replacen(&c[0], &shifted, 1);
        }
    }
    if let Some(c) = dmy.captures(original) {
        let d: u32 = c[1].parse().unwrap_or(1);
        let m: u32 = c[2].parse().unwrap_or(1);
        let y: i32 = c[3].parse().unwrap_or(1970);
        if let Some(iso) = shift_ymd(y, m, d, offset_days) {
            let sep = if original.contains('/') { '/' } else { '-' };
            let parts: Vec<&str> = iso.split('-').collect();
            if parts.len() == 3 {
                let yy: i32 = parts[0].parse().unwrap_or(y);
                let mo: u32 = parts[1].parse().unwrap_or(m);
                let day: u32 = parts[2].parse().unwrap_or(d);
                let repl = format!("{:02}{sep}{:02}{sep}{:04}", day, mo, yy);
                return original.replacen(&c[0], &repl, 1);
            }
        }
    }
    // Fallback: replace numeric spans with a synthetic ISO date anchored to offset.
    let base = civil_to_jdn(2020, 1, 1) + offset_days as i64;
    let (y, m, d) = jdn_to_civil(base);
    format!("{y:04}-{m:02}-{d:02}")
}

fn shift_ymd(year: i32, month: u32, day: u32, offset_days: i32) -> Option<String> {
    if !(1..=12).contains(&month) || day == 0 || day > 31 {
        return None;
    }
    let jdn = civil_to_jdn(year, month, day) + offset_days as i64;
    let (y, m, d) = jdn_to_civil(jdn);
    Some(format!("{y:04}-{m:02}-{d:02}"))
}

fn civil_to_jdn(year: i32, month: u32, day: u32) -> i64 {
    let a = (14_i64 - month as i64) / 12;
    let y = year as i64 + 4800 - a;
    let m = month as i64 + 12 * a - 3;
    day as i64 + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045
}

fn jdn_to_civil(jdn: i64) -> (i32, u32, u32) {
    let a = jdn + 32044;
    let b = (4 * a + 3) / 146097;
    let c = a - (146097 * b) / 4;
    let d = (4 * c + 3) / 1461;
    let e = c - (1461 * d) / 4;
    let m = (5 * e + 2) / 153;
    let day = (e - (153 * m + 2) / 5 + 1) as u32;
    let month = (m + 3 - 12 * (m / 10)) as u32;
    let year = (100 * b + d - 4800 + m / 10) as i32;
    (year, month, day)
}

fn gen_ipv4() -> String {
    let mut rng = rand::thread_rng();
    // Private-looking, non-reserved ranges for plausible substitutes.
    let a = rng.gen_range(10..=172);
    let b = rng.gen_range(0..=255);
    let c = rng.gen_range(0..=255);
    let d = rng.gen_range(1..=254);
    format!("{a}.{b}.{c}.{d}")
}

fn gen_ipv6() -> String {
    let mut rng = rand::thread_rng();
    let mut parts = Vec::with_capacity(8);
    for _ in 0..8 {
        parts.push(format!("{:x}", rng.gen_range(0x1000..=0xffff)));
    }
    parts.join(":")
}

fn gen_url_placeholder() -> String {
    let mut rng = rand::thread_rng();
    let base = SAFE_URL_DOMAINS[rng.gen_range(0..SAFE_URL_DOMAINS.len())];
    let id = uuid::Uuid::new_v4();
    format!("{base}/r/{id}")
}

fn gen_generic_token(original: &str) -> String {
    let mut rng = rand::thread_rng();
    let len = original.chars().count().max(4).min(48);
    let mut out = String::with_capacity(len);
    for ch in original.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_digit() {
                out.push(rng.gen_range(b'0'..=b'9') as char);
            } else if ch.is_ascii_lowercase() {
                out.push(rng.gen_range(b'a'..=b'z') as char);
            } else if ch.is_ascii_uppercase() {
                out.push(rng.gen_range(b'A'..=b'Z') as char);
            } else {
                out.push(ch);
            }
        } else {
            out.push(ch);
        }
    }
    if out.is_empty() {
        Alphanumeric.sample_string(&mut rng, len)
    } else {
        out
    }
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn pan_shape() {
        let p = gen_pan();
        assert_eq!(p.len(), 10);
        assert!(p[..5].chars().all(|c| c.is_ascii_uppercase()));
        assert!(p[5..9].chars().all(|c| c.is_ascii_digit()));
        assert!(p[9..].chars().all(|c| c.is_ascii_uppercase()));
    }
}
