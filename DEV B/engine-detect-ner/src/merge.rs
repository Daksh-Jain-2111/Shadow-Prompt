use std::collections::HashSet;

use crate::entity::{DetectionSource, EntitySpan};

fn priority(src: DetectionSource) -> u8 {
    match src {
        DetectionSource::UserRule => 3,
        DetectionSource::Regex => 2,
        DetectionSource::Keyword => 1,
        DetectionSource::NER => 0,
    }
}

fn better(a: &EntitySpan, b: &EntitySpan) -> bool {
    // true if a is a better choice than b
    let la = a.len();
    let lb = b.len();
    if la != lb {
        return la > lb;
    }
    let pa = priority(a.source);
    let pb = priority(b.source);
    if pa != pb {
        return pa > pb;
    }
    if a.start != b.start {
        return a.start < b.start;
    }
    if a.end != b.end {
        return a.end > b.end;
    }
    a.entity_type < b.entity_type
}

fn overlaps(a: &EntitySpan, b: &EntitySpan) -> bool {
    a.start < b.end && b.start < a.end
}

/// Merge spans using:
/// - longest match wins
/// - priority: UserRule > Regex > NER
/// - remove overlapping duplicates
///
/// The implementation treats any chain of overlaps as one group and selects the single best span.
pub fn merge_spans(mut spans: Vec<EntitySpan>) -> Vec<EntitySpan> {
    spans.retain(|s| s.start < s.end);
    if spans.is_empty() {
        return spans;
    }

    // Remove exact duplicates first (including source + entity_type).
    let mut seen: HashSet<(usize, usize, String, DetectionSource)> = HashSet::new();
    spans.retain(|s| {
        seen.insert((s.start, s.end, s.entity_type.clone(), s.source))
    });

    spans.sort_by(|a, b| {
        a.start
            .cmp(&b.start)
            .then_with(|| b.end.cmp(&a.end)) // longer first when same start
            .then_with(|| priority(b.source).cmp(&priority(a.source))) // higher priority first
            .then_with(|| a.entity_type.cmp(&b.entity_type))
    });

    let mut out: Vec<EntitySpan> = Vec::new();
    let mut cur_best: Option<EntitySpan> = None;
    let mut group_end: usize = 0;

    for s in spans.into_iter() {
        match cur_best.as_ref() {
            None => {
                group_end = s.end;
                cur_best = Some(s);
            }
            Some(best) => {
                if s.start < group_end {
                    if s.end > group_end {
                        group_end = s.end;
                    }
                    if better(&s, best) {
                        cur_best = Some(s);
                    }
                } else {
                    out.push(cur_best.take().unwrap());
                    group_end = s.end;
                    cur_best = Some(s);
                }
            }
        }
    }

    if let Some(last) = cur_best {
        out.push(last);
    }

    // Final safety: ensure non-overlap and stable ordering.
    out.sort_by_key(|s| s.start);
    let mut non_overlap: Vec<EntitySpan> = Vec::with_capacity(out.len());
    for s in out.into_iter() {
        if non_overlap.last().is_some_and(|p| overlaps(p, &s)) {
            // Should not happen, but if it does: keep the better one.
            let mut prev = non_overlap.pop().unwrap();
            if better(&s, &prev) {
                prev = s;
            }
            non_overlap.push(prev);
        } else {
            non_overlap.push(s);
        }
    }
    non_overlap
}

