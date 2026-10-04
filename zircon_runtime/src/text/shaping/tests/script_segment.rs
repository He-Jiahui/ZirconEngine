use super::{
    preferred_script_for_language, script_for_range, script_segments, ParagraphTextAnalysis,
};
use crate::text::{compiled_unicode_data_snapshot_id, FontScript, FontScriptTag, TextRange};
use unicode_script::Script;

#[test]
fn script_range_lookup_preserves_segment_boundaries() {
    let text = "abcمرحبا";
    let segments = script_segments(text, None);

    assert_eq!(
        script_for_range(&segments, TextRange { start: 0, end: 1 }).iso15924,
        "Latn"
    );
    assert_eq!(
        script_for_range(
            &segments,
            TextRange {
                start: 3,
                end: text.len(),
            },
        )
        .iso15924,
        "Arab"
    );
}

#[test]
fn script_extensions_follow_the_compatible_neighbor() {
    let text = "カ\u{30fc}A";

    let segments = script_segments(text, None);

    assert_eq!(segments.len(), 2);
    assert_eq!(segments[0].range, TextRange { start: 0, end: 6 });
    assert_eq!(segments[0].script.iso15924, "Kana");
    assert_eq!(segments[1].script.iso15924, "Latn");
}

#[test]
fn leading_common_and_inherited_characters_follow_the_first_specific_script() {
    let text = " \u{0301}مرحبا";

    let segments = script_segments(text, None);

    assert_eq!(segments.len(), 1);
    assert_eq!(
        segments[0].range,
        TextRange {
            start: 0,
            end: text.len()
        }
    );
    assert_eq!(segments[0].script.iso15924, "Arab");
}

#[test]
fn intermediate_and_trailing_common_punctuation_follow_the_previous_script() {
    let text = "abc-مرحبا!";
    let arabic_start = "abc-".len();

    let segments = script_segments(text, None);

    assert_eq!(segments.len(), 2);
    assert_eq!(
        segments[0].range,
        TextRange {
            start: 0,
            end: arabic_start,
        }
    );
    assert_eq!(segments[0].script.iso15924, "Latn");
    assert_eq!(
        segments[1].range,
        TextRange {
            start: arabic_start,
            end: text.len(),
        }
    );
    assert_eq!(segments[1].script.iso15924, "Arab");
}

#[test]
fn common_only_text_retains_a_common_script_segment() {
    let text = " ... ";

    let segments = script_segments(text, None);

    assert_eq!(segments.len(), 1);
    assert_eq!(
        segments[0].range,
        TextRange {
            start: 0,
            end: text.len()
        }
    );
    assert_eq!(segments[0].script.iso15924, "Zyyy");
}

#[test]
fn explicit_language_script_subtag_resolves_an_ambiguous_extension() {
    let segments = script_segments("\u{30fc}", Some("ja-Hira-JP"));

    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].script.iso15924, "Hira");
}

#[test]
fn private_use_language_subtags_do_not_impersonate_a_script_subtag() {
    assert_eq!(
        preferred_script_for_language(Some("ja-Hira-JP")),
        Some(Script::Hiragana)
    );
    assert_eq!(preferred_script_for_language(Some("ja-x-Kana")), None);
}

#[test]
fn paired_brackets_return_to_the_opening_context_script() {
    let text = "abc(مرحبا)def";
    let arabic_start = "abc(".len();
    let arabic_end = arabic_start + "مرحبا".len();

    let segments = script_segments(text, None);

    assert_eq!(segments.len(), 3);
    assert_eq!(segments[0].range.end, arabic_start);
    assert_eq!(segments[0].script.iso15924, "Latn");
    assert_eq!(
        segments[1].range,
        TextRange {
            start: arabic_start,
            end: arabic_end
        }
    );
    assert_eq!(segments[1].script.iso15924, "Arab");
    assert_eq!(segments[2].range.start, arabic_end);
    assert_eq!(segments[2].script.iso15924, "Latn");
}

#[test]
fn leading_and_nested_brackets_inherit_the_first_resolved_script() {
    let text = "([مرحبا])";

    let segments = script_segments(text, None);

    assert_eq!(segments.len(), 1);
    assert_eq!(
        segments[0].range,
        TextRange {
            start: 0,
            end: text.len()
        }
    );
    assert_eq!(segments[0].script.iso15924, "Arab");
}

#[test]
fn fallback_script_identity_uses_iso15924_instead_of_the_cluster_codepoint() {
    let first = "\u{13a0}";
    let second = "\u{13a1}";
    let first_analysis = ParagraphTextAnalysis::new(first, None);
    let second_analysis = ParagraphTextAnalysis::new(second, None);
    let expected = FontScript::Other(
        FontScriptTag::from_bytes(*b"Cher").expect("canonical Cherokee script tag"),
    );

    assert_eq!(
        first_analysis.font_script_for_range(TextRange {
            start: 0,
            end: first.len(),
        }),
        expected
    );
    assert_eq!(
        second_analysis.font_script_for_range(TextRange {
            start: 0,
            end: second.len(),
        }),
        expected
    );
}

#[test]
fn unassigned_codepoints_keep_a_typed_unknown_fallback_script() {
    let cluster = "\u{0378}";
    let analysis = ParagraphTextAnalysis::new(cluster, None);

    assert_eq!(
        analysis.font_script_for_range(TextRange {
            start: 0,
            end: cluster.len(),
        }),
        FontScript::Unknown
    );
}

#[test]
fn paragraph_emoji_presentation_uses_unicode_properties_and_selectors() {
    let text = "A☀☀\u{fe0f}😀\u{fe0e}\u{1f02c}";
    let analysis = ParagraphTextAnalysis::new(text, None);
    let latin = script_segments("A", None)[0].script;
    let sun_start = "A".len();
    let emoji_sun_start = sun_start + "☀".len();
    let text_face_start = emoji_sun_start + "☀\u{fe0f}".len();
    let unassigned_start = text_face_start + "😀\u{fe0e}".len();

    assert_eq!(
        analysis.shaped_script_for_range(TextRange {
            start: sun_start,
            end: emoji_sun_start,
        }),
        latin
    );
    assert_eq!(
        analysis
            .shaped_script_for_range(TextRange {
                start: emoji_sun_start,
                end: text_face_start,
            })
            .iso15924,
        "Zsye"
    );
    assert_eq!(
        analysis.shaped_script_for_range(TextRange {
            start: text_face_start,
            end: unassigned_start,
        }),
        latin
    );
    assert_eq!(
        analysis.shaped_script_for_range(TextRange {
            start: unassigned_start,
            end: text.len(),
        }),
        latin
    );
}

#[test]
fn paragraph_emoji_presentation_recognizes_keycaps_without_private_ranges() {
    let text = "A1\u{20e3}B";
    let analysis = ParagraphTextAnalysis::new(text, None);
    let keycap_start = "A".len();
    let keycap_end = keycap_start + "1\u{20e3}".len();

    assert_eq!(
        analysis
            .shaped_script_for_range(TextRange {
                start: keycap_start,
                end: keycap_end,
            })
            .iso15924,
        "Zsye"
    );
}

#[test]
fn paragraph_analysis_merges_adjacent_emoji_presentation_ranges() {
    let analysis = ParagraphTextAnalysis::new("\u{1f600}\u{1f601}A\u{1f602}", None);

    assert_eq!(analysis.emoji_presentation_ranges.len(), 2);
    assert_eq!(
        analysis.emoji_presentation_ranges[0],
        TextRange { start: 0, end: 8 }
    );
    assert_eq!(
        analysis.emoji_presentation_ranges[1],
        TextRange { start: 9, end: 13 }
    );
}

#[test]
fn paragraph_analysis_retains_request_unicode_snapshot_identity() {
    let current = compiled_unicode_data_snapshot_id();
    let next = current.with_generation_for_test(current.generation() + 1);
    let analysis = ParagraphTextAnalysis::for_snapshot("text", None, next);

    assert_eq!(analysis.unicode_data_snapshot(), next);
}

#[test]
fn empty_source_range_never_acquires_emoji_presentation() {
    let analysis = ParagraphTextAnalysis::new("\u{1f600}", None);

    assert_ne!(
        analysis
            .shaped_script_for_range(TextRange { start: 1, end: 1 })
            .iso15924,
        "Zsye"
    );
}

#[test]
fn script_range_lookup_does_not_restore_linear_find() {
    let source = include_str!("../script_segment.rs");
    let compact = source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();

    assert!(!compact.contains(concat!("segments.iter()", ".find")));
}
