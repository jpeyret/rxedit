use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::common::{self, GrepCommandQualifier, LineStatus, set_visibles};
use crate::utilities::vec_lines;

/// applies extra visibility logic according to what's in the qualifier:
/// expands context around hits using before/after radii anchored at hit positions.
pub(crate) fn post_search(
    qualifier: &GrepCommandQualifier,
    hits: &[usize],
    lines: Vec<LineStatus>,
) -> Vec<LineStatus> {
    let mut context_indices = HashSet::new();

    if qualifier.before > 0 {
        vec_lines::bounded_extend(&mut context_indices, hits, -qualifier.before, &lines);
    }
    if qualifier.after > 0 {
        vec_lines::bounded_extend(&mut context_indices, hits, qualifier.after, &lines);
    }

    set_visibles(lines, &context_indices)
}

pub(crate) fn has_valid_treesitter(hashtree: &HashMap<usize, common::Parsed>) -> bool {
    hashtree
        .values()
        .any(|parsed| parsed.as_declarations().is_some())
}

fn innermost_declaration_for_line(
    line_idx: usize,
    hashtree: &HashMap<usize, common::Parsed>,
) -> Option<&common::ParsedDeclarationsData> {
    hashtree
        .values()
        .filter_map(|parsed| parsed.as_declarations())
        .filter(|parsed| {
            let owner_start = parsed.lines_signature.0;
            let owner_end = parsed
                .lines_body
                .map(|(_, body_end)| body_end)
                .unwrap_or(parsed.end_line);
            owner_start <= line_idx && line_idx <= owner_end
        })
        .min_by(|left, right| {
            let left_start = left.lines_signature.0;
            let left_end = left
                .lines_body
                .map(|(_, body_end)| body_end)
                .unwrap_or(left.end_line);
            let left_len = left_end.saturating_sub(left_start);

            let right_start = right.lines_signature.0;
            let right_end = right
                .lines_body
                .map(|(_, body_end)| body_end)
                .unwrap_or(right.end_line);
            let right_len = right_end.saturating_sub(right_start);

            left_len
                .cmp(&right_len)
                .then_with(|| right_start.cmp(&left_start))
                .then(Ordering::Equal)
        })
}

pub(crate) fn show_owner_from_hits(
    hits: &[usize],
    lines: Vec<LineStatus>,
    hashtree: &HashMap<usize, common::Parsed>,
) -> Vec<LineStatus> {
    if hits.is_empty() || hashtree.is_empty() {
        return lines;
    }

    let mut owner_indices = HashSet::new();
    for hit in hits {
        if let Some(parsed) = innermost_declaration_for_line(*hit, hashtree) {
            let (sig_start, sig_end) = parsed.lines_signature;

            if lines.is_empty() {
                continue;
            }

            let clamped_start = sig_start.min(lines.len() - 1);
            let clamped_end = sig_end.min(lines.len() - 1);
            for idx in clamped_start..=clamped_end {
                owner_indices.insert(idx);
            }
        }
    }

    if owner_indices.is_empty() {
        return lines;
    }

    set_visibles(lines, &owner_indices)
}
