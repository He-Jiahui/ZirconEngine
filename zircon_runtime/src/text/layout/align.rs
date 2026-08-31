use unicode_script::{Script, UnicodeScript};
use unicode_segmentation::UnicodeSegmentation;

use crate::text::{TextJoiningTypeMap, compiled_joining_type_map};

const JUSTIFY_EPSILON: f32 = 0.01;

pub(crate) fn justify_line_advances(
    text: &str,
    advances: &[f32],
    natural_width: f32,
    target_width: f32,
) -> Option<Vec<f32>> {
    let extra = target_width - natural_width;
    if extra <= JUSTIFY_EPSILON {
        return None;
    }

    let graphemes = text.graphemes(true).collect::<Vec<_>>();
    if graphemes.len() != advances.len() || graphemes.len() < 2 {
        return None;
    }

    let opportunities = justification_opportunities(&graphemes);
    if opportunities.is_empty() {
        return None;
    }

    let mut adjusted = advances.to_vec();
    let per_opportunity = extra / opportunities.len() as f32;
    let mut assigned = 0.0;
    for (position, index) in opportunities.iter().copied().enumerate() {
        let delta = if position + 1 == opportunities.len() {
            extra - assigned
        } else {
            per_opportunity
        };
        adjusted[index] = (adjusted[index] + delta).max(0.0);
        assigned += delta;
    }

    Some(adjusted)
}

/// Returns logical byte offsets immediately after Arabic joining pairs that are candidates for a
/// virtual tatweel. The shaping backend must still validate that a candidate is safe for the
/// selected face and language before materializing it.
pub(crate) fn arabic_kashida_insertion_offsets(text: &str) -> Vec<usize> {
    let joining_types = compiled_joining_type_map();
    arabic_kashida_insertion_offsets_with_map(text, joining_types)
}

fn arabic_kashida_insertion_offsets_with_map(
    text: &str,
    joining_types: TextJoiningTypeMap,
) -> Vec<usize> {
    let mut offsets = Vec::new();
    let mut previous = None;
    for (start, grapheme) in text.grapheme_indices(true) {
        if previous.is_some_and(|left| is_arabic_kashida_pair(left, grapheme, joining_types)) {
            offsets.push(start);
        }
        previous = Some(grapheme);
    }
    offsets
}

/// Returns at most `limit` evenly distributed insertion offsets without allocating one entry for
/// every joining pair in a long line.
pub(crate) fn arabic_kashida_insertion_offsets_bounded(text: &str, limit: usize) -> Vec<usize> {
    if limit == 0 {
        return Vec::new();
    }

    let joining_types = compiled_joining_type_map();
    let pair_count = arabic_kashida_pair_count(text, joining_types);
    if pair_count <= limit {
        return arabic_kashida_insertion_offsets_with_map(text, joining_types);
    }

    let mut offsets = Vec::with_capacity(limit);
    let mut next_slot = 0;
    let mut next_pair_index = Some(evenly_spaced_pair_index(next_slot, pair_count, limit));
    let mut pair_index = 0;
    let mut previous = None;
    for (start, grapheme) in text.grapheme_indices(true) {
        if previous.is_some_and(|left| is_arabic_kashida_pair(left, grapheme, joining_types)) {
            if next_pair_index == Some(pair_index) {
                offsets.push(start);
                next_slot += 1;
                next_pair_index = if next_slot < limit {
                    Some(evenly_spaced_pair_index(next_slot, pair_count, limit))
                } else {
                    None
                };
            }
            pair_index += 1;
        }
        previous = Some(grapheme);
    }
    offsets
}

fn arabic_kashida_pair_count(text: &str, joining_types: TextJoiningTypeMap) -> usize {
    let mut count = 0;
    let mut previous = None;
    for (_, grapheme) in text.grapheme_indices(true) {
        if previous.is_some_and(|left| is_arabic_kashida_pair(left, grapheme, joining_types)) {
            count += 1;
        }
        previous = Some(grapheme);
    }
    count
}

fn evenly_spaced_pair_index(slot: usize, pair_count: usize, slot_count: usize) -> usize {
    let bucket_start =
        slot * (pair_count / slot_count) + slot * (pair_count % slot_count) / slot_count;
    let next_slot = slot + 1;
    let bucket_end =
        next_slot * (pair_count / slot_count) + next_slot * (pair_count % slot_count) / slot_count;
    bucket_start + (bucket_end - bucket_start) / 2
}

fn justification_opportunities(graphemes: &[&str]) -> Vec<usize> {
    let Some((content_start, content_end)) = content_grapheme_range(graphemes) else {
        return Vec::new();
    };

    let mut opportunities = Vec::new();
    let joining_types = compiled_joining_type_map();
    for index in content_start..content_end.saturating_sub(1) {
        if is_word_space(graphemes[index]) {
            opportunities.push(index);
            continue;
        }
        if is_cjk_justifiable_pair(graphemes[index], graphemes[index + 1]) {
            opportunities.push(index);
            continue;
        }
        if is_arabic_kashida_pair(graphemes[index], graphemes[index + 1], joining_types) {
            if let Some(tatweel_index) =
                arabic_tatweel_opportunity_index(graphemes, index, joining_types)
            {
                opportunities.push(tatweel_index);
            }
        }
    }
    opportunities
}

fn arabic_tatweel_opportunity_index(
    graphemes: &[&str],
    index: usize,
    joining_types: TextJoiningTypeMap,
) -> Option<usize> {
    if is_tatweel(graphemes[index + 1], joining_types) {
        return Some(index + 1);
    }
    None
}

fn content_grapheme_range(graphemes: &[&str]) -> Option<(usize, usize)> {
    let start = graphemes
        .iter()
        .position(|grapheme| !is_word_space(grapheme))?;
    let end = graphemes
        .iter()
        .rposition(|grapheme| !is_word_space(grapheme))?
        + 1;
    Some((start, end))
}

fn is_word_space(grapheme: &str) -> bool {
    matches!(grapheme, " " | "\u{3000}")
}

fn is_cjk_justifiable_pair(left: &str, right: &str) -> bool {
    cjk_char(left).is_some() && cjk_char(right).is_some()
}

fn is_arabic_kashida_pair(left: &str, right: &str, joining_types: TextJoiningTypeMap) -> bool {
    let Some(left) = arabic_grapheme_base(left, joining_types) else {
        return false;
    };
    let Some(right) = arabic_grapheme_base(right, joining_types) else {
        return false;
    };

    joining_types
        .get(left)
        .joins_with_following_logical_character()
        && joining_types
            .get(right)
            .joins_with_preceding_logical_character()
}

fn arabic_grapheme_base(grapheme: &str, joining_types: TextJoiningTypeMap) -> Option<char> {
    let mut chars = grapheme.chars();
    let base = chars.next()?;
    ((base.script() == Script::Arabic || base == '\u{0640}')
        && chars.all(|ch| is_arabic_grapheme_continuation(ch, joining_types)))
    .then_some(base)
}

fn is_arabic_grapheme_continuation(ch: char, joining_types: TextJoiningTypeMap) -> bool {
    joining_types.get(ch).is_transparent() || matches!(ch, '\u{200d}')
}

fn cjk_char(grapheme: &str) -> Option<char> {
    single_char(grapheme).filter(|ch| {
        matches!(
            *ch as u32,
            0x3040..=0x30FF
                | 0x31F0..=0x31FF
                | 0x3400..=0x4DBF
                | 0x4E00..=0x9FFF
                | 0xAC00..=0xD7AF
                | 0xF900..=0xFAFF
                | 0x20000..=0x2FA1F
        )
    })
}

fn single_char(grapheme: &str) -> Option<char> {
    let mut chars = grapheme.chars();
    let ch = chars.next()?;
    chars.next().is_none().then_some(ch)
}

fn is_tatweel(grapheme: &str, joining_types: TextJoiningTypeMap) -> bool {
    matches!(
        arabic_grapheme_base(grapheme, joining_types),
        Some('\u{0640}')
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arabic_kashida_offsets_follow_each_joining_pair() {
        assert_eq!(arabic_kashida_insertion_offsets("سلام"), vec![2, 4]);
    }

    #[test]
    fn arabic_kashida_offsets_keep_combining_marks_with_their_base_grapheme() {
        assert_eq!(arabic_kashida_insertion_offsets("سَلَام"), vec![4, 8]);
    }

    #[test]
    fn arabic_kashida_offsets_keep_an_explicit_joiner_with_its_base_grapheme() {
        assert_eq!(arabic_kashida_insertion_offsets("س\u{200d}لام"), vec![5, 7]);
    }

    #[test]
    fn arabic_kashida_offsets_do_not_cross_an_explicit_non_joiner() {
        assert_eq!(arabic_kashida_insertion_offsets("س\u{200c}لام"), vec![7]);
    }

    #[test]
    fn arabic_kashida_uses_unicode_joining_type_beyond_the_retired_ranges() {
        assert_eq!(arabic_kashida_insertion_offsets("س\u{0870}"), vec![2]);
    }

    #[test]
    fn arabic_kashida_does_not_admit_other_joining_scripts() {
        assert!(arabic_kashida_insertion_offsets("\u{10acd}\u{10acd}").is_empty());
    }

    #[test]
    fn bounded_arabic_kashida_offsets_sample_long_lines_without_collecting_every_pair() {
        assert_eq!(
            arabic_kashida_insertion_offsets_bounded("سلمسلمسلمسلم", 3),
            vec![4, 12, 20]
        );
    }

    #[test]
    fn bounded_arabic_kashida_offsets_allow_no_insertions() {
        assert!(arabic_kashida_insertion_offsets_bounded("سلام", 0).is_empty());
    }

    #[test]
    fn arabic_justify_requires_a_materialized_tatweel_for_extra_advance() {
        assert!(justify_line_advances("سلام", &[10.0; 4], 40.0, 41.0).is_none());
    }

    #[test]
    fn arabic_justify_expands_a_marked_materialized_tatweel() {
        let adjusted = justify_line_advances("سـَلام", &[10.0; 5], 50.0, 55.0)
            .expect("marked tatweel remains a concrete justify target");

        assert_eq!(adjusted[1], 15.0);
    }
}
