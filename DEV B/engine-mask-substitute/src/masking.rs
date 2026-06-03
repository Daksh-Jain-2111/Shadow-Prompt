//! Apply replacements from right to left on the **original** string using non-overlapping spans.

use crate::session_map::SessionMap;
use crate::Detection;

fn is_strictly_inside(inner: &Detection, outer: &Detection) -> bool {
    outer.start <= inner.start
        && inner.end <= outer.end
        && (outer.start < inner.start || inner.end < outer.end)
}

/// Drop detections that sit strictly inside another span so nested matches mask the whole secret.
fn drop_strictly_contained<'a>(valid: &[&'a Detection]) -> Vec<&'a Detection> {
    let mut out: Vec<&Detection> = Vec::new();
    for d in valid {
        let contained = valid.iter().any(|other| {
            !std::ptr::eq(*d, *other) && is_strictly_inside(d, other)
        });
        if !contained {
            out.push(d);
        }
    }
    out
}

/// Mask sensitive spans in `text` using `session` for stable substitutes.
///
/// Detections are applied on **original** indices: drop strictly nested spans, then pick a
/// non-overlapping set greedy from the end (descending `start`), then stitch left-to-right.
pub fn mask_text(text: &str, detections: &[Detection], session: &mut SessionMap) -> String {
    let mut valid: Vec<&Detection> = detections.iter().filter(|d| d.is_valid).collect();
    if valid.is_empty() {
        return text.to_string();
    }

    valid = drop_strictly_contained(&valid);
    valid.sort_by_key(|d| d.start);
    let mut accepted: Vec<&Detection> = Vec::new();
    let mut cursor_end = text.len();

    // Greedy from the right: largest `start` first among candidates that end before `cursor_end`.
    for det in valid.iter().rev() {
        if det.end <= cursor_end {
            accepted.push(det);
            cursor_end = det.start;
        }
    }

    accepted.sort_by_key(|d| d.start);

    let mut out = String::with_capacity(text.len());
    let mut pos = 0usize;
    for det in &accepted {
        if det.start < pos || det.end > text.len() || det.start > det.end {
            continue;
        }
        out.push_str(&text[pos..det.start]);
        let sub = session.get_or_create(&det.entity_type, &det.value);
        out.push_str(&sub);
        pos = det.end;
    }
    out.push_str(&text[pos..]);
    out
}
