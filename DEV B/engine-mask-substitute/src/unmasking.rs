//! Restore model output: longest-first exact replace, then token-scoped fuzzy Levenshtein.

use crate::generators::normalize_entity_type;
use crate::session_map::SessionMap;

/// Replace known substitutes with their originals. Longer masked tokens first.
pub fn unmask_text(text: &str, session: &SessionMap) -> String {
    if session.reverse.is_empty() {
        return text.to_string();
    }

    let mut pairs: Vec<(&String, &String)> = session.reverse.iter().collect();
    pairs.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    let mut out = text.to_string();
    for (masked, original) in pairs {
        if masked.is_empty() {
            continue;
        }
        out = out.replace(masked.as_str(), original.as_str());
    }
    out
}

fn fuzzy_threshold(subst_len: usize) -> usize {
    (subst_len / 8).max(2)
}

fn levenshtein(a: &str, b: &str) -> usize {
    let ab: Vec<char> = a.chars().collect();
    let bb: Vec<char> = b.chars().collect();
    let n = ab.len();
    let m = bb.len();
    if n == 0 {
        return m;
    }
    if m == 0 {
        return n;
    }
    let mut prev: Vec<usize> = (0..=m).collect();
    let mut curr = vec![0usize; m + 1];
    for ca in &ab {
        curr[0] = prev[0] + 1;
        for j in 0..m {
            let cost = usize::from(ca != &bb[j]);
            curr[j + 1] = (curr[j] + 1)
                .min(prev[j + 1] + 1)
                .min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[m]
}

fn utf16_prefix_len(s: &str, char_end: usize) -> usize {
    s.chars()
        .take(char_end)
        .map(|c| c.len_utf16())
        .sum()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FuzzySpan {
    pub utf16_start: usize,
    pub utf16_end: usize,
    pub masked: String,
    pub revealed: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnmaskFuzzyResult {
    pub text: String,
    pub fuzzy_spans: Vec<FuzzySpan>,
}

fn token_char_spans(s: &str) -> Vec<(usize, usize)> {
    let chars: Vec<char> = s.chars().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < chars.len() {
        let c = chars[i];
        if is_token_char(c) {
            let start = i;
            i += 1;
            while i < chars.len() && is_token_char(chars[i]) {
                i += 1;
            }
            if i - start >= 3 {
                out.push((start, i));
            }
        } else {
            i += 1;
        }
    }
    out
}

fn is_token_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '@' | '.' | '_' | '+' | '-')
}

fn types_compatible(subst_entity: &str, token: &str, subst: &str) -> bool {
    let t = normalize_entity_type(subst_entity);
    match t.as_ref() {
        "AADHAAR" => {
            token.chars().all(|c| c.is_ascii_digit())
                && subst.chars().all(|c| c.is_ascii_digit())
                && (10..=14).contains(&token.len())
        }
        "PAN" => {
            subst.len() == 10
                && token.len() <= 12
                && token.chars().filter(|c| c.is_ascii_alphabetic()).count() >= 4
        }
        "PHONE" => token.chars().filter(|c| c.is_ascii_digit()).count() >= 8,
        "NAME" | "PERSON" => token.chars().any(|c| c.is_ascii_alphabetic()),
        _ => true,
    }
}

fn collect_fuzzy_replacements(exact: &str, session: &SessionMap) -> Vec<(usize, usize, String, String)> {
    let mut subst_pairs: Vec<(&String, &String)> = session.reverse.iter().collect();
    subst_pairs.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    let chars: Vec<char> = exact.chars().collect();
    let mut used = vec![false; chars.len()];

    let mut chosen: Vec<(usize, usize, String, String)> = Vec::new();

    let mut token_spans = token_char_spans(exact);
    token_spans.sort_by_key(|(a, b)| b - a);

    for (cs, ce) in token_spans {
        if used[cs..ce].iter().any(|u| *u) {
            continue;
        }
        let token: String = chars[cs..ce].iter().collect();
        let mut best: Option<(&str, usize, String, String)> = None;
        for (subst, orig) in &subst_pairs {
            let thr = fuzzy_threshold(subst.len());
            let len_diff = token.len().abs_diff(subst.len());
            if len_diff > thr {
                continue;
            }
            let dist = levenshtein(&token, subst);
            if dist == 0 || dist > thr {
                continue;
            }
            let et = session
                .substitute_entity
                .get(*subst)
                .map(|s| s.as_str())
                .unwrap_or("");
            if !types_compatible(et, &token, subst) {
                continue;
            }
            let better = match &best {
                None => true,
                Some((bsub, bd, _, _)) => {
                    dist < *bd || (dist == *bd && subst.len() > bsub.len())
                }
            };
            if better {
                best = Some((subst, dist, orig.to_string(), subst.to_string()));
            }
        }

        if let Some((_subst, _dist, orig, masked)) = best {
            for u in &mut used[cs..ce] {
                *u = true;
            }
            chosen.push((cs, ce, orig, masked));
        }
    }

    chosen.sort_by_key(|(cs, _, _, _)| *cs);
    chosen
}

/// Exact unmask first, then fuzzy token restore with per-type guards.
pub fn unmask_exact_then_fuzzy(input: &str, session: &SessionMap) -> UnmaskFuzzyResult {
    let exact = unmask_text(input, session);
    if session.reverse.is_empty() {
        return UnmaskFuzzyResult {
            text: exact,
            fuzzy_spans: vec![],
        };
    }

    let spans = collect_fuzzy_replacements(&exact, session);
    if spans.is_empty() {
        return UnmaskFuzzyResult {
            text: exact,
            fuzzy_spans: vec![],
        };
    }

    let chars: Vec<char> = exact.chars().collect();
    let mut out = String::new();
    let mut fuzzy_spans = Vec::new();
    let mut cursor = 0usize;

    for (cs, ce, orig, masked) in spans {
        if cursor < cs {
            out.push_str(&chars[cursor..cs].iter().collect::<String>());
        }
        let char_start = out.chars().count();
        let u16_start = utf16_prefix_len(&out, char_start);
        out.push_str(&orig);
        let char_end = out.chars().count();
        let u16_end = utf16_prefix_len(&out, char_end);
        fuzzy_spans.push(FuzzySpan {
            utf16_start: u16_start,
            utf16_end: u16_end,
            masked,
            revealed: orig,
        });
        cursor = ce;
    }
    if cursor < chars.len() {
        out.push_str(&chars[cursor..].iter().collect::<String>());
    }

    UnmaskFuzzyResult {
        text: out,
        fuzzy_spans,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SessionMap;

    #[test]
    fn fuzzy_restores_typo_substitute() {
        let mut session = SessionMap::new();
        session.insert_mapping("PAN", "ABCDE1234F", "ZXCVB5678Q");
        let model = "Please check ZXCVB5670Q today";
        let r = unmask_exact_then_fuzzy(model, &session);
        assert!(r.text.contains("ABCDE1234F"));
        assert!(!r.fuzzy_spans.is_empty());
    }

    #[test]
    fn fuzzy_name_person_requires_letter_in_token() {
        let mut session = SessionMap::new();
        session.insert_mapping("NAME", "Rahul", "Arjun");
        let model = "code 9999999999 onlydigits";
        let r = unmask_exact_then_fuzzy(model, &session);
        assert!(
            !r.fuzzy_spans.iter().any(|s| s.revealed == "Rahul"),
            "digit-only tokens must not fuzzy-restore NAME/PERSON"
        );
    }

    #[test]
    fn fuzzy_phone_typo_can_restore() {
        let mut session = SessionMap::new();
        session.insert_mapping("PHONE", "9876543210", "8765432109");
        let model = "sms 8765432119";
        let r = unmask_exact_then_fuzzy(model, &session);
        assert!(r.text.contains("9876543210"));
    }
}
