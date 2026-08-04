use crate::text::layout::trailing_wrap_space_byte_len;
use zircon_runtime_interface::ui::surface::{
    UiResolvedTextRun, UiTextDirection, UiTextRange, UiTextRunKind,
};

use super::direction::resolve_direction;
use super::range_mapping::source_subrange;

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

/// Adds rendered-only text at an existing visual boundary. The virtual run is deliberately
/// zero-width in source space so selection, IME, and hit testing continue to address the source.
pub(super) fn insert_virtual_text(
    line: &mut CandidateLine,
    visual_offset: usize,
    virtual_text: &str,
) -> bool {
    if virtual_text.is_empty()
        || visual_offset > line.text.len()
        || !line.text.is_char_boundary(visual_offset)
    {
        return false;
    }

    let mut text = String::with_capacity(line.text.len() + virtual_text.len());
    let mut runs = Vec::with_capacity(line.runs.len().saturating_add(2));
    let mut inserted = false;
    for run in &line.runs {
        if !inserted
            && run.visual_range.start <= visual_offset
            && visual_offset <= run.visual_range.end
        {
            let local_offset = visual_offset.saturating_sub(run.visual_range.start);
            if !run.text.is_char_boundary(local_offset) {
                return false;
            }
            let source_offset = if run.source_range.start == run.source_range.end {
                run.source_range.start
            } else if run.source_range.end.saturating_sub(run.source_range.start) == run.text.len()
            {
                run.source_range.start + local_offset
            } else {
                return false;
            };
            push_run_fragment(&mut text, &mut runs, run, 0, local_offset);
            let visual_start = text.len();
            text.push_str(virtual_text);
            runs.push(UiResolvedTextRun {
                kind: run.kind,
                text: virtual_text.to_string(),
                source_range: UiTextRange {
                    start: source_offset,
                    end: source_offset,
                },
                visual_range: UiTextRange {
                    start: visual_start,
                    end: text.len(),
                },
                direction: resolve_direction(virtual_text, UiTextDirection::Auto),
            });
            push_run_fragment(&mut text, &mut runs, run, local_offset, run.text.len());
            inserted = true;
        } else {
            push_run_fragment(&mut text, &mut runs, run, 0, run.text.len());
        }
    }
    if !inserted || text.len() != line.text.len().saturating_add(virtual_text.len()) {
        return false;
    }

    line.text = text;
    line.runs = runs;
    true
}

fn push_run_fragment(
    text: &mut String,
    runs: &mut Vec<UiResolvedTextRun>,
    run: &UiResolvedTextRun,
    start: usize,
    end: usize,
) {
    let Some(fragment) = run.text.get(start..end) else {
        return;
    };
    if fragment.is_empty() {
        return;
    }
    let visual_start = text.len();
    text.push_str(fragment);
    runs.push(UiResolvedTextRun {
        kind: run.kind,
        text: fragment.to_string(),
        source_range: source_subrange(run.source_range, run.text.len(), start, end),
        visual_range: UiTextRange {
            start: visual_start,
            end: text.len(),
        },
        direction: run.direction,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_text_keeps_later_candidate_offsets_and_source_anchors() {
        let mut line = CandidateLine::empty();
        append_segment(
            &mut line,
            UiTextRunKind::Plain,
            "سلام",
            UiTextRange { start: 0, end: 8 },
        );

        assert!(insert_virtual_text(&mut line, 2, "ـ"));
        assert!(insert_virtual_text(&mut line, 6, "ـ"));

        assert_eq!(line.text, "سـلـام");
        assert_eq!(line.source_range, UiTextRange { start: 0, end: 8 });
        assert_eq!(
            line.runs
                .iter()
                .filter(|run| run.source_range.start == run.source_range.end)
                .map(|run| run.source_range.start)
                .collect::<Vec<_>>(),
            vec![2, 4]
        );
    }
}
