use crate::text::SharedTextLayoutSession;
use crate::text::layout::{
    ELLIPSIS, EllipsisPlacement, measure_line_width_with_provider,
    measured_grapheme_widths_with_provider, retained_grapheme_counts,
    trim_end_ellipsis_trailing_graphemes,
};
use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextRun, UiTextDirection, UiTextOverflow, UiTextRange, UiTextRunKind,
};

use crate::text::text_style;
use super::candidate_line::{CandidateLine, append_segment};
use super::direction::resolve_direction;
use super::range_mapping::source_subrange;
use super::wrapping::line_text_fits_with_provider;

const ELLIPSIS_FIT_EPSILON: f32 = 0.01;

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

pub(super) fn ellipsize_line_with_provider(
    line: &mut CandidateLine,
    max_width: f32,
    style: &UiResolvedStyle,
    overflow: UiTextOverflow,
    provider: &mut SharedTextLayoutSession,
) {
    let neutral_style = text_style(style);
    let mut advances = measured_grapheme_widths_with_provider(&line.text, &neutral_style, provider);
    let ellipsis_advance = measure_line_width_with_provider(ELLIPSIS, &neutral_style, provider);
    force_ellipsize_line_with_advances(line, &mut advances, max_width, ellipsis_advance, overflow);
}

pub(super) fn ellipsize_line_with_advances(
    line: &mut CandidateLine,
    advances: &mut Vec<f32>,
    max_width: f32,
    ellipsis_advance: f32,
    overflow: UiTextOverflow,
) {
    ellipsize_line_with_advances_inner(
        line,
        advances,
        max_width,
        ellipsis_advance,
        overflow,
        false,
    );
}

pub(super) fn force_ellipsize_line_with_advances(
    line: &mut CandidateLine,
    advances: &mut Vec<f32>,
    max_width: f32,
    ellipsis_advance: f32,
    overflow: UiTextOverflow,
) {
    ellipsize_line_with_advances_inner(line, advances, max_width, ellipsis_advance, overflow, true);
}

fn ellipsize_line_with_advances_inner(
    line: &mut CandidateLine,
    advances: &mut Vec<f32>,
    max_width: f32,
    ellipsis_advance: f32,
    overflow: UiTextOverflow,
    force: bool,
) {
    let graphemes = line
        .text
        .grapheme_indices(true)
        .map(|(start, grapheme)| (start, start + grapheme.len()))
        .collect::<Vec<_>>();
    if graphemes.len() != advances.len()
        || (!force
            && advances.iter().copied().sum::<f32>() <= max_width.max(0.0) + ELLIPSIS_FIT_EPSILON)
    {
        return;
    }

    let available =
        (max_width.max(0.0) + ELLIPSIS_FIT_EPSILON - ellipsis_advance.max(0.0)).max(0.0);
    let placement = ellipsis_placement(overflow);
    let (mut prefix_count, suffix_count) =
        retained_grapheme_counts(&line.text, &graphemes, advances, available, placement);
    trim_end_ellipsis_trailing_graphemes(&line.text, &graphemes, &mut prefix_count, placement);
    let prefix_end = if prefix_count == 0 {
        0
    } else {
        graphemes[prefix_count - 1].1
    };
    let suffix_start = graphemes
        .get(graphemes.len().saturating_sub(suffix_count))
        .map(|(start, _)| *start)
        .unwrap_or(line.text.len());

    let mut text = String::new();
    let mut runs = Vec::new();
    if prefix_end > 0 {
        push_ellipsis_range(&mut text, &mut runs, line, 0, prefix_end);
    }
    push_ellipsis_run(
        &mut text,
        &mut runs,
        ellipsis_source_offset(line, placement),
    );
    if suffix_start < line.text.len() {
        push_ellipsis_range(&mut text, &mut runs, line, suffix_start, line.text.len());
    }

    let mut retained_advances = Vec::with_capacity(prefix_count + suffix_count + 1);
    retained_advances.extend_from_slice(&advances[..prefix_count]);
    retained_advances.push(ellipsis_advance.max(0.0));
    if suffix_count > 0 {
        retained_advances.extend_from_slice(&advances[advances.len() - suffix_count..]);
    }
    line.text = text;
    line.runs = runs;
    line.ellipsized = true;
    *advances = retained_advances;
}

fn ellipsis_placement(overflow: UiTextOverflow) -> EllipsisPlacement {
    match overflow {
        UiTextOverflow::EllipsisWord => EllipsisPlacement::EndWord,
        UiTextOverflow::EllipsisStart => EllipsisPlacement::Start,
        UiTextOverflow::EllipsisMiddle => EllipsisPlacement::Middle,
        _ => EllipsisPlacement::End,
    }
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
    push_or_merge_ellipsis_run(
        runs,
        UiResolvedTextRun {
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
        },
    );
}

fn push_ellipsis_range(
    text: &mut String,
    runs: &mut Vec<UiResolvedTextRun>,
    line: &CandidateLine,
    start: usize,
    end: usize,
) {
    let mut visual_cursor = 0;
    for run in &line.runs {
        let run_start = visual_cursor;
        let run_end = run_start + run.text.len();
        visual_cursor = run_end;
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
    push_or_merge_ellipsis_run(
        runs,
        UiResolvedTextRun {
            kind: run.kind,
            text: fragment.to_string(),
            source_range,
            visual_range: UiTextRange {
                start: visual_start,
                end: text.len(),
            },
            direction,
        },
    );
}

fn push_or_merge_ellipsis_run(runs: &mut Vec<UiResolvedTextRun>, run: UiResolvedTextRun) {
    if let Some(previous) = runs.last_mut() {
        let preserves_semantic_boundary = previous.text.contains(ELLIPSIS)
            || run.text.contains(ELLIPSIS)
            || previous.text.contains('\u{fffc}')
            || run.text.contains('\u{fffc}');
        if !preserves_semantic_boundary
            && previous.kind == run.kind
            && previous.direction == run.direction
            && previous.source_range.end == run.source_range.start
            && previous.visual_range.end == run.visual_range.start
        {
            previous.text.push_str(&run.text);
            previous.source_range.end = run.source_range.end;
            previous.visual_range.end = run.visual_range.end;
            return;
        }
    }
    runs.push(run);
}

pub(super) fn line_overflows_horizontally_with_provider(
    line: &CandidateLine,
    max_width: f32,
    style: &UiResolvedStyle,
    provider: &mut SharedTextLayoutSession,
) -> bool {
    !line.text.is_empty() && !line_text_fits_with_provider(&line.text, max_width, style, provider)
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
