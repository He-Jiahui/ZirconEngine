use crate::graphics::text::layout::{
    ellipsize_text, measure_line_width_with_provider, EllipsisPlacement, EllipsisSegment, ELLIPSIS,
};
use crate::graphics::text::shaping::TextShapeRunProvider;
use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextRun, UiTextDirection, UiTextOverflow, UiTextRange, UiTextRunKind,
};

use super::candidate_line::{append_segment, CandidateLine};
use super::direction::resolve_direction;
use super::range_mapping::source_subrange;
use super::wrapping::line_text_fits_with_provider;

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

pub(super) fn ellipsize_line_with_provider<P>(
    line: &mut CandidateLine,
    max_width: f32,
    style: &UiResolvedStyle,
    overflow: UiTextOverflow,
    provider: &mut P,
) where
    P: TextShapeRunProvider + ?Sized,
{
    let mut text = String::new();
    let mut runs = Vec::new();
    let placement = match overflow {
        UiTextOverflow::EllipsisWord => EllipsisPlacement::EndWord,
        UiTextOverflow::EllipsisStart => EllipsisPlacement::Start,
        UiTextOverflow::EllipsisMiddle => EllipsisPlacement::Middle,
        _ => EllipsisPlacement::End,
    };
    let segments = ellipsize_text(&line.text, max_width, placement, |candidate| {
        measure_line_width_with_provider(candidate, style, provider)
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
        || (!force && advances.iter().copied().sum::<f32>() <= max_width.max(0.0))
    {
        return;
    }

    let available = (max_width.max(0.0) - ellipsis_advance.max(0.0)).max(0.0);
    let (prefix_count, suffix_count) =
        retained_grapheme_counts(&line.text, &graphemes, advances, available, overflow);
    let prefix_end = graphemes
        .get(prefix_count.saturating_sub(1))
        .map(|(_, end)| *end)
        .unwrap_or_default();
    let suffix_start = graphemes
        .get(graphemes.len().saturating_sub(suffix_count))
        .map(|(start, _)| *start)
        .unwrap_or(line.text.len());

    let mut text = String::new();
    let mut runs = Vec::new();
    let placement = ellipsis_placement(overflow);
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

fn retained_grapheme_counts(
    text: &str,
    graphemes: &[(usize, usize)],
    advances: &[f32],
    available: f32,
    overflow: UiTextOverflow,
) -> (usize, usize) {
    match ellipsis_placement(overflow) {
        EllipsisPlacement::Start => (0, fitting_suffix_count(advances, available)),
        EllipsisPlacement::Middle => {
            let prefix_budget = available * 0.5;
            let prefix = fitting_prefix_count(advances, prefix_budget);
            let prefix_width = advances[..prefix].iter().copied().sum::<f32>();
            let suffix =
                fitting_suffix_count(&advances[prefix..], (available - prefix_width).max(0.0))
                    .min(advances.len().saturating_sub(prefix));
            (prefix, suffix)
        }
        EllipsisPlacement::EndWord => {
            let fitted = fitting_prefix_count(advances, available);
            let fitted_end = graphemes
                .get(fitted.saturating_sub(1))
                .map(|(_, end)| *end)
                .unwrap_or_default();
            let word_end = text
                .split_word_bound_indices()
                .filter_map(|(start, word)| {
                    let end = start + word.len();
                    (!word.trim().is_empty() && end <= fitted_end).then_some(end)
                })
                .last()
                .unwrap_or(fitted_end);
            (
                graphemes
                    .iter()
                    .take(fitted)
                    .take_while(|(_, end)| *end <= word_end)
                    .count(),
                0,
            )
        }
        EllipsisPlacement::End => (fitting_prefix_count(advances, available), 0),
    }
}

fn fitting_prefix_count(advances: &[f32], available: f32) -> usize {
    let mut width = 0.0;
    advances
        .iter()
        .take_while(|advance| {
            let fits = width + **advance <= available;
            if fits {
                width += **advance;
            }
            fits
        })
        .count()
}

fn fitting_suffix_count(advances: &[f32], available: f32) -> usize {
    let mut width = 0.0;
    advances
        .iter()
        .rev()
        .take_while(|advance| {
            let fits = width + **advance <= available;
            if fits {
                width += **advance;
            }
            fits
        })
        .count()
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

pub(super) fn line_overflows_horizontally_with_provider<P>(
    line: &CandidateLine,
    max_width: f32,
    style: &UiResolvedStyle,
    provider: &mut P,
) -> bool
where
    P: TextShapeRunProvider + ?Sized,
{
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
