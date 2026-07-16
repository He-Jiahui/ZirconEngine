use crate::text::layout::trailing_wrap_space_byte_len;
use zircon_runtime_interface::ui::surface::{
    UiResolvedTextRun, UiTextDirection, UiTextRange, UiTextRunKind,
};

use super::direction::resolve_direction;

#[derive(Clone, Debug)]
pub(super) struct CandidateLine {
    pub text: String,
    pub source_range: UiTextRange,
    pub runs: Vec<UiResolvedTextRun>,
    pub pending_break_suffix: Option<PendingBreakSuffix>,
    pub ellipsized: bool,
}

impl CandidateLine {
    pub(super) fn empty() -> Self {
        Self {
            text: String::new(),
            source_range: UiTextRange::default(),
            runs: Vec::new(),
            pending_break_suffix: None,
            ellipsized: false,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct PendingBreakSuffix {
    pub kind: UiTextRunKind,
    pub text: &'static str,
    pub source_range: UiTextRange,
}

pub(super) fn append_segment(
    current: &mut CandidateLine,
    kind: UiTextRunKind,
    text: &str,
    source_range: UiTextRange,
) {
    if text.is_empty() {
        return;
    }
    let visual_start = current.text.len();
    current.text.push_str(text);
    let visual_end = current.text.len();
    if current.runs.is_empty() {
        current.source_range.start = source_range.start;
    }
    current.source_range.end = source_range.end;
    current.runs.push(UiResolvedTextRun {
        kind,
        text: text.to_string(),
        source_range,
        visual_range: UiTextRange {
            start: visual_start,
            end: visual_end,
        },
        direction: resolve_direction(text, UiTextDirection::Auto),
    });
}

pub(super) fn push_current_line(lines: &mut Vec<CandidateLine>, current: &mut CandidateLine) {
    if !current.text.is_empty() || !lines.is_empty() {
        current.pending_break_suffix = None;
        lines.push(std::mem::replace(current, CandidateLine::empty()));
    }
}

pub(super) fn push_wrapped_line(lines: &mut Vec<CandidateLine>, current: &mut CandidateLine) {
    append_pending_break_suffix(current);
    push_current_line(lines, current);
}

pub(super) fn append_pending_break_suffix(current: &mut CandidateLine) {
    let Some(suffix) = current.pending_break_suffix.take() else {
        return;
    };
    append_segment(current, suffix.kind, suffix.text, suffix.source_range);
}

pub(super) fn trim_word_break_trailing_spaces(line: &mut CandidateLine) {
    let mut spaces_to_trim = trailing_wrap_space_byte_len(&line.text);
    while spaces_to_trim > 0 {
        let Some(last_run) = line.runs.last_mut() else {
            break;
        };
        if !last_run.text.ends_with(' ') {
            break;
        }
        line.text.pop();
        last_run.text.pop();
        last_run.source_range.end = last_run.source_range.end.saturating_sub(1);
        last_run.visual_range.end = last_run.visual_range.end.saturating_sub(1);
        if last_run.text.is_empty() {
            line.runs.pop();
        }
        spaces_to_trim -= 1;
    }
    line.source_range.end = line
        .runs
        .last()
        .map(|run| run.source_range.end)
        .unwrap_or(line.source_range.start);
}
