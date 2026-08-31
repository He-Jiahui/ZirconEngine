use crate::text::TextRange;
use unicode_bidi::{BidiDataSource, HardcodedBidiData};
use unicode_script::{Script, ScriptExtension, UnicodeScript};
use unicode_segmentation::UnicodeSegmentation;

use crate::text::language::{TextLanguageScriptSubtag, text_language_script_subtag};
use crate::text::{FontScript, Iso15924Tag, ShapedGlyphScript};
use crate::text::{UnicodeDataSnapshotId, compiled_unicode_data_snapshot_id};

use super::emoji_presentation::cluster_uses_emoji_presentation;

const EMOJI_SCRIPT_TAG: Iso15924Tag = Iso15924Tag::EMOJI;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ScriptSegment {
    pub range: TextRange,
    pub script: ShapedGlyphScript,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ParagraphTextAnalysis {
    scripts: Vec<ScriptSegment>,
    emoji_presentation_ranges: Vec<TextRange>,
    unicode_data_snapshot: UnicodeDataSnapshotId,
}

impl ParagraphTextAnalysis {
    pub(crate) fn new(text: &str, language: Option<&str>) -> Self {
        Self::for_snapshot(
            text,
            text_language_script_subtag(language),
            compiled_unicode_data_snapshot_id(),
        )
    }

    pub(crate) fn for_snapshot(
        text: &str,
        explicit_language_script: Option<TextLanguageScriptSubtag>,
        unicode_data_snapshot: UnicodeDataSnapshotId,
    ) -> Self {
        #[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
        let profile_started = super::analysis_profile::start_build();
        let analysis = Self {
            scripts: script_segments_with_preferred_script(
                text,
                unicode_script_for_language(explicit_language_script),
            ),
            emoji_presentation_ranges: emoji_presentation_ranges(text),
            unicode_data_snapshot,
        };
        #[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
        super::analysis_profile::record_script_emoji_build(text.len(), profile_started);
        analysis
    }

    pub(crate) const fn unicode_data_snapshot(&self) -> UnicodeDataSnapshotId {
        self.unicode_data_snapshot
    }

    pub(crate) fn script_for_range(&self, range: TextRange) -> ShapedGlyphScript {
        script_for_range(&self.scripts, range)
    }

    pub(crate) fn shaped_script_for_range(&self, range: TextRange) -> ShapedGlyphScript {
        if range_overlaps_any(&self.emoji_presentation_ranges, range) {
            ShapedGlyphScript {
                iso15924: EMOJI_SCRIPT_TAG,
            }
        } else {
            self.script_for_range(range)
        }
    }

    pub(crate) fn font_script_for_range(&self, range: TextRange) -> FontScript {
        FontScript::from_iso15924_tag(self.shaped_script_for_range(range).iso15924.as_str())
    }
}

#[derive(Clone, Copy)]
struct PairedBracketContext {
    opening: char,
    script: Option<Script>,
}

pub(crate) fn script_segments(text: &str, language: Option<&str>) -> Vec<ScriptSegment> {
    script_segments_with_preferred_script(text, preferred_script_for_language(language))
}

fn script_segments_with_preferred_script(
    text: &str,
    preferred_script: Option<Script>,
) -> Vec<ScriptSegment> {
    let mut segments = Vec::new();
    let mut current_start = 0;
    let mut current_candidates = ScriptExtension::default();
    let mut current_primary = None;
    let mut previous_script = None;
    let mut has_current = false;
    let mut bracket_stack = Vec::<PairedBracketContext>::new();
    let mut unresolved_bracket_start = None;

    for (start, ch) in text.char_indices() {
        let primary = script_for_char(ch);
        let mut extensions = script_extension_for_char(ch);
        let paired_bracket = is_common_like(primary)
            .then(|| HardcodedBidiData.bidi_matched_opening_bracket(ch))
            .flatten();
        let mut matched_bracket_index = None;

        if let Some(bracket) = paired_bracket {
            if bracket.is_open {
                let script = contextual_script(
                    current_candidates,
                    current_primary,
                    preferred_script,
                    previous_script,
                );
                if script.is_none() && unresolved_bracket_start.is_none() {
                    unresolved_bracket_start = Some(bracket_stack.len());
                }
                bracket_stack.push(PairedBracketContext {
                    opening: bracket.opening,
                    script,
                });
            } else if let Some(index) = bracket_stack
                .iter()
                .rposition(|entry| entry.opening == bracket.opening)
            {
                let opening_script = bracket_stack[index].script.or_else(|| {
                    contextual_script(
                        current_candidates,
                        current_primary,
                        preferred_script,
                        previous_script,
                    )
                });
                if let Some(opening_script) = opening_script {
                    extensions = opening_script.into();
                }
                matched_bracket_index = Some(index);
            } else {
                bracket_stack.clear();
                unresolved_bracket_start = None;
            }
        }

        if !has_current {
            current_start = start;
            current_candidates = extensions;
            current_primary = specific_script(primary);
            has_current = true;
        } else {
            match compatible_intersection(current_candidates, extensions) {
                Some(intersection) => {
                    current_candidates = intersection;
                    if current_primary.is_none()
                        && specific_script(primary)
                            .is_some_and(|script| intersection.contains_script(script))
                    {
                        current_primary = specific_script(primary);
                    }
                }
                None => {
                    let resolved = resolve_script(
                        current_candidates,
                        current_primary,
                        preferred_script,
                        previous_script,
                    );
                    push_segment(&mut segments, current_start, start, resolved);
                    resolve_pending_brackets(
                        &mut bracket_stack,
                        &mut unresolved_bracket_start,
                        resolved,
                    );
                    previous_script = specific_script(resolved);
                    current_start = start;
                    current_candidates = extensions;
                    current_primary = specific_script(primary)
                        .filter(|script| extensions.contains_script(*script));
                }
            }
        }

        if let Some(resolved) = contextual_script(
            current_candidates,
            current_primary,
            preferred_script,
            previous_script,
        ) {
            resolve_pending_brackets(&mut bracket_stack, &mut unresolved_bracket_start, resolved);
        }
        if let Some(index) = matched_bracket_index {
            truncate_bracket_stack(&mut bracket_stack, &mut unresolved_bracket_start, index);
        }
    }

    if has_current {
        let resolved = resolve_script(
            current_candidates,
            current_primary,
            preferred_script,
            previous_script,
        );
        push_segment(&mut segments, current_start, text.len(), resolved);
    }

    segments
}

fn emoji_presentation_ranges(text: &str) -> Vec<TextRange> {
    let mut ranges = Vec::<TextRange>::new();
    for (start, cluster) in text.grapheme_indices(true) {
        if !cluster_uses_emoji_presentation(cluster) {
            continue;
        }
        let end = start + cluster.len();
        if let Some(previous) = ranges.last_mut().filter(|range| range.end == start) {
            previous.end = end;
        } else {
            ranges.push(TextRange { start, end });
        }
    }
    ranges
}

fn range_overlaps_any(ranges: &[TextRange], range: TextRange) -> bool {
    if range.start >= range.end {
        return false;
    }
    ranges
        .get(ranges.partition_point(|candidate| candidate.end <= range.start))
        .is_some_and(|candidate| candidate.start < range.end)
}

pub(crate) fn script_for_range(segments: &[ScriptSegment], range: TextRange) -> ShapedGlyphScript {
    let midpoint = range.start + range.end.saturating_sub(range.start) / 2;
    segments
        .get(segments.partition_point(|segment| segment.range.end <= midpoint))
        .filter(|segment| midpoint >= segment.range.start && midpoint < segment.range.end)
        .or_else(|| {
            segments
                .get(segments.partition_point(|segment| segment.range.end <= range.start))
                .filter(|segment| range.start < segment.range.end)
        })
        .map(|segment| segment.script)
        .unwrap_or_default()
}

fn push_segment(segments: &mut Vec<ScriptSegment>, start: usize, end: usize, script: Script) {
    if start >= end {
        return;
    }
    let script = shaped_script(script);
    if let Some(last) = segments.last_mut() {
        if last.script == script && last.range.end == start {
            last.range.end = end;
            return;
        }
    }
    segments.push(ScriptSegment {
        range: TextRange { start, end },
        script,
    });
}

fn compatible_intersection(
    left: ScriptExtension,
    right: ScriptExtension,
) -> Option<ScriptExtension> {
    let intersection = left.intersection(right);
    if !intersection.is_empty() {
        Some(intersection)
    } else if left.is_empty() && right.is_empty() {
        Some(left)
    } else {
        None
    }
}

fn is_common_like(script: Script) -> bool {
    matches!(script, Script::Common | Script::Inherited | Script::Unknown)
}

fn script_for_char(ch: char) -> Script {
    ch.script()
}

fn script_extension_for_char(ch: char) -> ScriptExtension {
    ch.script_extension()
}

fn specific_script(script: Script) -> Option<Script> {
    (!is_common_like(script)).then_some(script)
}

fn contextual_script(
    candidates: ScriptExtension,
    primary: Option<Script>,
    preferred: Option<Script>,
    previous: Option<Script>,
) -> Option<Script> {
    if candidates.is_empty() || candidates.is_common() || candidates.is_inherited() {
        return None;
    }
    if candidates.len() == 1 {
        return candidates.iter().next().and_then(specific_script);
    }
    previous
        .filter(|script| candidates.contains_script(*script))
        .or_else(|| preferred.filter(|script| candidates.contains_script(*script)))
        .or_else(|| primary.filter(|script| candidates.contains_script(*script)))
}

fn resolve_script(
    candidates: ScriptExtension,
    primary: Option<Script>,
    preferred: Option<Script>,
    previous: Option<Script>,
) -> Script {
    contextual_script(candidates, primary, preferred, previous).unwrap_or_else(|| {
        if candidates.is_empty() {
            Script::Unknown
        } else if candidates.is_inherited() {
            Script::Inherited
        } else if candidates.is_common() {
            Script::Common
        } else {
            candidates
                .iter()
                .find_map(specific_script)
                .unwrap_or(Script::Unknown)
        }
    })
}

fn resolve_pending_brackets(
    stack: &mut [PairedBracketContext],
    unresolved_start: &mut Option<usize>,
    script: Script,
) {
    let Some(script) = specific_script(script) else {
        return;
    };
    let Some(start) = unresolved_start.take() else {
        return;
    };
    for entry in &mut stack[start..] {
        debug_assert!(entry.script.is_none());
        entry.script = Some(script);
    }
}

fn truncate_bracket_stack(
    stack: &mut Vec<PairedBracketContext>,
    unresolved_start: &mut Option<usize>,
    len: usize,
) {
    stack.truncate(len);
    if unresolved_start.is_some_and(|start| start >= len) {
        *unresolved_start = None;
    }
}

fn preferred_script_for_language(language: Option<&str>) -> Option<Script> {
    unicode_script_for_language(text_language_script_subtag(language))
}

fn unicode_script_for_language(script: Option<TextLanguageScriptSubtag>) -> Option<Script> {
    script
        .and_then(|script| script.as_str().and_then(Script::from_short_name))
        .and_then(specific_script)
}

fn shaped_script(script: Script) -> ShapedGlyphScript {
    ShapedGlyphScript {
        iso15924: match Iso15924Tag::parse(script.short_name()) {
            Some(tag) => tag,
            None => Iso15924Tag::COMMON,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ParagraphTextAnalysis, preferred_script_for_language, script_for_range, script_segments,
    };
    use crate::text::{FontScript, FontScriptTag, TextRange, compiled_unicode_data_snapshot_id};
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
        let source = include_str!("script_segment.rs");
        let compact = source
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();

        assert!(!compact.contains(concat!("segments.iter()", ".find")));
    }
}
