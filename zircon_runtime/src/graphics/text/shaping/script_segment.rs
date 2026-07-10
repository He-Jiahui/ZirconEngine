use unicode_script::{Script, UnicodeScript};
use zircon_runtime_interface::ui::surface::UiTextRange;

use crate::core::framework::render::{FontScript, ShapedGlyphScript};

const EMOJI_SCRIPT_TAG: &str = "Zsye";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ScriptSegment {
    pub range: UiTextRange,
    pub script: ShapedGlyphScript,
}

pub(crate) fn script_segments(text: &str) -> Vec<ScriptSegment> {
    let mut segments = Vec::new();
    let mut current_start = 0;
    let mut current_script = Script::Common;
    let mut has_current = false;
    let mut pending_common_start = None;
    let mut pending_common_end = None;

    for (start, ch) in text.char_indices() {
        let end = start + ch.len_utf8();
        let script = script_for_char(ch);

        if is_common_like(script) {
            if !has_current {
                current_start = start;
                current_script = script;
                has_current = true;
            }
            if pending_common_start.is_none() {
                pending_common_start = Some(start);
            }
            pending_common_end = Some(end);
            continue;
        }

        if !has_current {
            current_start = start;
            current_script = script;
            has_current = true;
            pending_common_start = None;
            pending_common_end = None;
            continue;
        }

        if is_common_like(current_script) {
            current_script = script;
            pending_common_start = None;
            pending_common_end = None;
            continue;
        }

        if same_script(current_script, script) {
            pending_common_start = None;
            pending_common_end = None;
            current_script = script;
        } else {
            let split_at = pending_common_end.unwrap_or(start);
            push_segment(&mut segments, current_start, split_at, current_script);
            current_start = split_at;
            current_script = script;
            pending_common_start = None;
            pending_common_end = None;
        }
    }

    if has_current {
        let end = pending_common_end.unwrap_or(text.len());
        push_segment(&mut segments, current_start, end, current_script);
    }

    segments
}

pub(crate) fn script_for_range(
    segments: &[ScriptSegment],
    range: UiTextRange,
) -> ShapedGlyphScript {
    let midpoint = range.start + range.end.saturating_sub(range.start) / 2;
    segments
        .iter()
        .find(|segment| midpoint >= segment.range.start && midpoint < segment.range.end)
        .or_else(|| {
            segments
                .iter()
                .find(|segment| range.start < segment.range.end)
        })
        .map(|segment| segment.script.clone())
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
        range: UiTextRange { start, end },
        script,
    });
}

fn same_script(left: Script, right: Script) -> bool {
    is_common_like(left) || is_common_like(right) || left == right
}

fn is_common_like(script: Script) -> bool {
    matches!(script, Script::Common | Script::Inherited | Script::Unknown)
}

fn script_for_char(ch: char) -> Script {
    if is_emoji_script(ch) {
        Script::Common
    } else {
        ch.script()
    }
}

fn shaped_script(script: Script) -> ShapedGlyphScript {
    ShapedGlyphScript {
        iso15924: script.short_name().to_string(),
    }
}

fn is_emoji_script(ch: char) -> bool {
    matches!(
        ch as u32,
        0x1F000..=0x1FAFF | 0x2600..=0x27BF | 0x200D | 0xFE0E | 0xFE0F
    )
}

pub(crate) fn shaped_script_for_cluster(
    cluster_text: &str,
    fallback: ShapedGlyphScript,
) -> ShapedGlyphScript {
    if cluster_text.chars().any(is_emoji_script) {
        ShapedGlyphScript {
            iso15924: EMOJI_SCRIPT_TAG.to_string(),
        }
    } else {
        fallback
    }
}

pub(crate) fn font_script_for_cluster(cluster: &str) -> FontScript {
    let fallback = cluster
        .chars()
        .map(script_for_char)
        .find(|script| !is_common_like(*script))
        .unwrap_or(Script::Common);
    match shaped_script_for_cluster(cluster, shaped_script(fallback))
        .iso15924
        .as_str()
    {
        "Latn" => FontScript::Latin,
        "Cyrl" => FontScript::Cyrillic,
        "Grek" => FontScript::Greek,
        "Hani" => FontScript::Han,
        "Hira" => FontScript::Hiragana,
        "Kana" => FontScript::Katakana,
        "Hang" => FontScript::Hangul,
        "Arab" => FontScript::Arabic,
        "Hebr" => FontScript::Hebrew,
        "Deva" => FontScript::Devanagari,
        _ => cluster
            .chars()
            .find(|codepoint| !codepoint.is_whitespace())
            .map(|codepoint| FontScript::Other(codepoint as u32))
            .unwrap_or(FontScript::Other(0)),
    }
}
