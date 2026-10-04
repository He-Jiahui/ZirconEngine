use crate::text::TextRange;
use unicode_bidi::{BidiDataSource, HardcodedBidiData};
use unicode_script::{Script, ScriptExtension, UnicodeScript};
use unicode_segmentation::UnicodeSegmentation;

use crate::text::language::{text_language_script_subtag, TextLanguageScriptSubtag};
use crate::text::{compiled_unicode_data_snapshot_id, UnicodeDataSnapshotId};
use crate::text::{FontScript, Iso15924Tag, ShapedGlyphScript};

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
#[path = "tests/script_segment.rs"]
mod tests;
