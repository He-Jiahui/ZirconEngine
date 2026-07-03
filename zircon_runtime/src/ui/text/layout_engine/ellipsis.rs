use crate::graphics::text::layout::{
    ellipsize_text, measure_line_width, EllipsisPlacement, EllipsisSegment, ELLIPSIS,
};
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextRun, UiTextDirection, UiTextOverflow, UiTextRange, UiTextRunKind,
};

use super::candidate_line::{append_segment, CandidateLine};
use super::direction::resolve_direction;
use super::range_mapping::source_subrange;
use super::wrapping::line_text_fits;

pub(super) fn merge_clipped_lines_for_tail_preserving_ellipsis(
    lines: &mut Vec<CandidateLine>,
    line_capacity: usize,
) {
    if line_capacity == 0 || lines.len() <= line_capacity {
        return;
    }

    let clipped = lines.split_off(line_capacity);
    let Some(last_visible) = lines.last_mut() else {
        lines.extend(clipped);
        return;
    };
    for line in clipped {
        for run in line.runs {
            append_segment(last_visible, run.kind, &run.text, run.source_range);
        }
    }
}

pub(super) fn ellipsize_line(
    line: &mut CandidateLine,
    max_width: f32,
    style: &UiResolvedStyle,
    overflow: UiTextOverflow,
) {
    let mut text = String::new();
    let mut runs = Vec::new();
    let placement = match overflow {
        UiTextOverflow::EllipsisWord => EllipsisPlacement::EndWord,
        UiTextOverflow::EllipsisStart => EllipsisPlacement::Start,
        UiTextOverflow::EllipsisMiddle => EllipsisPlacement::Middle,
        _ => EllipsisPlacement::End,
    };
    let segments = ellipsize_text(&line.text, max_width, placement, |candidate| {
        measure_line_width(candidate, style)
    });

    for segment in segments {
        match segment {
            EllipsisSegment::Text { start, end } => {
                push_ellipsis_range(&mut text, &mut runs, line, start, end);
            }
            EllipsisSegment::Ellipsis => {
                push_ellipsis_run(
                    &mut text,
                    &mut runs,
                    ellipsis_source_offset(line, placement),
                );
            }
        }
    }

    line.text = text;
    line.runs = runs;
    line.ellipsized = true;
}

fn ellipsis_source_offset(line: &CandidateLine, placement: EllipsisPlacement) -> usize {
    match placement {
        EllipsisPlacement::Start => line.source_range.start,
        EllipsisPlacement::End | EllipsisPlacement::EndWord | EllipsisPlacement::Middle => {
            line.source_range.end
        }
    }
}

fn push_ellipsis_run(text: &mut String, runs: &mut Vec<UiResolvedTextRun>, source_offset: usize) {
    let visual_start = text.len();
    text.push_str(ELLIPSIS);
    runs.push(UiResolvedTextRun {
        kind: UiTextRunKind::Plain,
        text: ELLIPSIS.to_string(),
        source_range: UiTextRange {
            start: source_offset,
            end: source_offset,
        },
        visual_range: UiTextRange {
            start: visual_start,
            end: text.len(),
        },
        direction: resolve_direction(ELLIPSIS, UiTextDirection::Auto),
    });
}

fn push_ellipsis_range(
    text: &mut String,
    runs: &mut Vec<UiResolvedTextRun>,
    line: &CandidateLine,
    start: usize,
    end: usize,
) {
    for run in &line.runs {
        let run_start = run.visual_range.start;
        let run_end = run.visual_range.end;
        let local_start = start.max(run_start);
        let local_end = end.min(run_end);
        if local_start >= local_end {
            continue;
        }
        push_ellipsis_fragment(
            text,
            runs,
            run,
            local_start - run_start,
            local_end - run_start,
        );
    }
}

fn push_ellipsis_fragment(
    text: &mut String,
    runs: &mut Vec<UiResolvedTextRun>,
    run: &UiResolvedTextRun,
    start: usize,
    end: usize,
) {
    if start >= end {
        return;
    }
    let fragment = &run.text[start..end];
    let visual_start = text.len();
    text.push_str(fragment);
    let source_range = source_subrange(run.source_range, run.text.len(), start, end);
    let direction = resolve_direction(fragment, UiTextDirection::Auto);
    if let Some(previous) = runs.last_mut() {
        if previous.kind == run.kind
            && previous.direction == direction
            && previous.source_range.end == source_range.start
            && previous.visual_range.end == visual_start
        {
            previous.text.push_str(fragment);
            previous.source_range.end = source_range.end;
            previous.visual_range.end = text.len();
            return;
        }
    }
    runs.push(UiResolvedTextRun {
        kind: run.kind,
        text: fragment.to_string(),
        source_range,
        visual_range: UiTextRange {
            start: visual_start,
            end: text.len(),
        },
        direction,
    });
}

pub(super) fn line_overflows_horizontally(
    line: &CandidateLine,
    max_width: f32,
    style: &UiResolvedStyle,
) -> bool {
    !line.text.is_empty() && !line_text_fits(&line.text, max_width, style)
}

pub(super) fn is_ellipsis_overflow(overflow: UiTextOverflow) -> bool {
    matches!(
        overflow,
        UiTextOverflow::Ellipsis
            | UiTextOverflow::EllipsisWord
            | UiTextOverflow::EllipsisStart
            | UiTextOverflow::EllipsisMiddle
    )
}
