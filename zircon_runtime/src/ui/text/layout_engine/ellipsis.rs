use crate::text::layout::{
    measure_line_width_with_provider, measured_grapheme_widths_with_provider,
    retained_grapheme_counts, trim_end_ellipsis_trailing_graphemes, EllipsisPlacement,
    LogicalVirtualFragmentRole, ELLIPSIS,
};
use crate::text::layout_geometry::finite_sum;
use crate::text::shaping::{TextLayoutOutcome, TextShapingOutcome};
use crate::text::SharedTextLayoutSession;
use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextRun, UiTextDirection, UiTextOverflow, UiTextRange, UiTextRunKind,
};

use super::candidate_line::{append_segment, CandidateLine, VirtualTextSourceReceipt};
use super::direction::resolve_direction;
use super::range_mapping::source_subrange;
use super::wrapping::line_text_fits_with_provider;
use crate::text::text_style;

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
        let visual_origin = last_visible.text.len();
        let virtual_source_receipts = line.virtual_source_receipts.clone();
        for run in line.runs {
            append_segment(last_visible, run.kind, &run.text, run.source_range);
        }
        last_visible
            .virtual_source_receipts
            .extend(
                virtual_source_receipts
                    .into_iter()
                    .map(|owner| VirtualTextSourceReceipt {
                        visual_range: UiTextRange {
                            start: visual_origin.saturating_add(owner.visual_range.start),
                            end: visual_origin.saturating_add(owner.visual_range.end),
                        },
                        style_source_range: owner.style_source_range,
                        replaced_source_range: owner.replaced_source_range,
                        virtual_role: owner.virtual_role,
                    }),
            );
    }
}

pub(super) fn ellipsize_line_with_provider(
    line: &mut CandidateLine,
    max_width: f32,
    style: &UiResolvedStyle,
    overflow: UiTextOverflow,
    provider: &mut SharedTextLayoutSession,
) -> TextLayoutOutcome<()> {
    let neutral_style = text_style(style);
    let mut advances =
        match measured_grapheme_widths_with_provider(&line.text, &neutral_style, provider) {
            TextShapingOutcome::Ready(advances) => advances,
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        };
    let ellipsis_advance =
        match measure_line_width_with_provider(ELLIPSIS, &neutral_style, provider) {
            TextShapingOutcome::Ready(width) => width,
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        };
    force_ellipsize_line_with_advances(line, &mut advances, max_width, ellipsis_advance, overflow);
    TextShapingOutcome::Ready(())
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
        None,
    );
}

pub(super) fn ellipsize_line_with_advances_and_style_owner(
    line: &mut CandidateLine,
    advances: &mut Vec<f32>,
    max_width: f32,
    ellipsis_advance: f32,
    overflow: UiTextOverflow,
    style_owner_source_range: Option<UiTextRange>,
) {
    ellipsize_line_with_advances_inner(
        line,
        advances,
        max_width,
        ellipsis_advance,
        overflow,
        false,
        style_owner_source_range,
    );
}

pub(super) fn force_ellipsize_line_with_advances(
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
        true,
        None,
    );
}

pub(super) fn force_ellipsize_line_with_advances_and_style_owner(
    line: &mut CandidateLine,
    advances: &mut Vec<f32>,
    max_width: f32,
    ellipsis_advance: f32,
    overflow: UiTextOverflow,
    style_owner_source_range: Option<UiTextRange>,
) {
    ellipsize_line_with_advances_inner(
        line,
        advances,
        max_width,
        ellipsis_advance,
        overflow,
        true,
        style_owner_source_range,
    );
}

fn ellipsize_line_with_advances_inner(
    line: &mut CandidateLine,
    advances: &mut Vec<f32>,
    max_width: f32,
    ellipsis_advance: f32,
    overflow: UiTextOverflow,
    force: bool,
    style_owner_source_range: Option<UiTextRange>,
) {
    let graphemes = line
        .text
        .grapheme_indices(true)
        .map(|(start, grapheme)| (start, start + grapheme.len()))
        .collect::<Vec<_>>();
    if graphemes.len() != advances.len()
        || (!force
            && finite_sum(advances.iter().copied()) <= max_width.max(0.0) + ELLIPSIS_FIT_EPSILON)
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
    let style_owner = style_owner_source_range
        .and_then(|source_range| style_owner_for_source_range(line, source_range))
        .or_else(|| ellipsis_style_owner(line, prefix_end, suffix_start, placement));

    let mut text = String::new();
    let mut runs = Vec::new();
    let mut virtual_source_receipts = Vec::new();
    if prefix_end > 0 {
        push_ellipsis_range(
            &mut text,
            &mut runs,
            &mut virtual_source_receipts,
            line,
            0,
            prefix_end,
        );
    }
    let marker_visual_range = push_ellipsis_run(
        &mut text,
        &mut runs,
        ellipsis_source_offset(line, placement),
        style_owner,
    );
    if suffix_start < line.text.len() {
        push_ellipsis_range(
            &mut text,
            &mut runs,
            &mut virtual_source_receipts,
            line,
            suffix_start,
            line.text.len(),
        );
    }
    if let Some(style_owner) = style_owner {
        virtual_source_receipts.push(VirtualTextSourceReceipt {
            visual_range: marker_visual_range,
            style_source_range: style_owner.source_range,
            replaced_source_range: single_omitted_source_range(line.source_range, &runs),
            virtual_role: LogicalVirtualFragmentRole::Ellipsis,
        });
        virtual_source_receipts.sort_by_key(|receipt| receipt.visual_range.start);
    }

    let mut retained_advances = Vec::with_capacity(prefix_count + suffix_count + 1);
    retained_advances.extend_from_slice(&advances[..prefix_count]);
    retained_advances.push(ellipsis_advance.max(0.0));
    if suffix_count > 0 {
        retained_advances.extend_from_slice(&advances[advances.len() - suffix_count..]);
    }
    line.text = text;
    line.runs = runs;
    line.virtual_source_receipts = virtual_source_receipts;
    line.ellipsized = true;
    *advances = retained_advances;
}

pub(super) fn ellipsis_style_owner_source_range(
    line: &CandidateLine,
    advances: &[f32],
    max_width: f32,
    overflow: UiTextOverflow,
) -> Option<UiTextRange> {
    let graphemes = line
        .text
        .grapheme_indices(true)
        .map(|(start, grapheme)| (start, start + grapheme.len()))
        .collect::<Vec<_>>();
    if graphemes.len() != advances.len() || graphemes.is_empty() {
        return None;
    }
    let placement = ellipsis_placement(overflow);
    let (mut prefix_count, suffix_count) = retained_grapheme_counts(
        &line.text,
        &graphemes,
        advances,
        max_width.max(0.0) + ELLIPSIS_FIT_EPSILON,
        placement,
    );
    trim_end_ellipsis_trailing_graphemes(&line.text, &graphemes, &mut prefix_count, placement);
    let prefix_end = prefix_count
        .checked_sub(1)
        .and_then(|index| graphemes.get(index))
        .map(|(_, end)| *end)
        .unwrap_or_default();
    let suffix_start = graphemes
        .get(graphemes.len().saturating_sub(suffix_count))
        .map(|(start, _)| *start)
        .unwrap_or(line.text.len());
    ellipsis_style_owner(line, prefix_end, suffix_start, placement).map(|owner| owner.source_range)
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

fn push_ellipsis_run(
    text: &mut String,
    runs: &mut Vec<UiResolvedTextRun>,
    source_offset: usize,
    style_owner: Option<EllipsisStyleOwner>,
) -> UiTextRange {
    let visual_start = text.len();
    text.push_str(ELLIPSIS);
    let visual_range = UiTextRange {
        start: visual_start,
        end: text.len(),
    };
    push_or_merge_ellipsis_run(
        runs,
        UiResolvedTextRun {
            kind: style_owner.map_or(UiTextRunKind::Plain, |owner| owner.kind),
            text: ELLIPSIS.to_string(),
            source_range: UiTextRange {
                start: source_offset,
                end: source_offset,
            },
            visual_range,
            direction: resolve_direction(ELLIPSIS, UiTextDirection::Auto),
        },
    );
    visual_range
}

fn push_ellipsis_range(
    text: &mut String,
    runs: &mut Vec<UiResolvedTextRun>,
    virtual_source_receipts: &mut Vec<VirtualTextSourceReceipt>,
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
            virtual_source_receipts,
            &line.virtual_source_receipts,
            run,
            local_start - run_start,
            local_end - run_start,
        );
    }
}

fn push_ellipsis_fragment(
    text: &mut String,
    runs: &mut Vec<UiResolvedTextRun>,
    virtual_source_receipts: &mut Vec<VirtualTextSourceReceipt>,
    source_virtual_source_receipts: &[VirtualTextSourceReceipt],
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
    let source_visual_start = run.visual_range.start.saturating_add(start);
    let source_visual_end = run.visual_range.start.saturating_add(end);
    virtual_source_receipts.extend(source_virtual_source_receipts.iter().filter_map(|owner| {
        (source_visual_start <= owner.visual_range.start
            && owner.visual_range.end <= source_visual_end)
            .then_some(VirtualTextSourceReceipt {
                visual_range: UiTextRange {
                    start: visual_start
                        .saturating_add(owner.visual_range.start - source_visual_start),
                    end: visual_start.saturating_add(owner.visual_range.end - source_visual_start),
                },
                style_source_range: owner.style_source_range,
                replaced_source_range: owner.replaced_source_range,
                virtual_role: owner.virtual_role,
            })
    }));
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

fn single_omitted_source_range(
    line_source_range: UiTextRange,
    retained_runs: &[UiResolvedTextRun],
) -> Option<UiTextRange> {
    let mut ranges = retained_runs
        .iter()
        .filter_map(|run| {
            let start = run.source_range.start.max(line_source_range.start);
            let end = run.source_range.end.min(line_source_range.end);
            (start < end).then_some(UiTextRange { start, end })
        })
        .collect::<Vec<_>>();
    ranges.sort_by_key(|range| (range.start, range.end));

    let mut gaps = Vec::new();
    let mut cursor = line_source_range.start;
    for range in ranges {
        if cursor < range.start {
            gaps.push(UiTextRange {
                start: cursor,
                end: range.start,
            });
        }
        cursor = cursor.max(range.end);
    }
    if cursor < line_source_range.end {
        gaps.push(UiTextRange {
            start: cursor,
            end: line_source_range.end,
        });
    }
    (gaps.len() == 1).then(|| gaps[0])
}

#[derive(Clone, Copy)]
struct EllipsisStyleOwner {
    source_range: UiTextRange,
    kind: UiTextRunKind,
}

fn style_owner_for_source_range(
    line: &CandidateLine,
    source_range: UiTextRange,
) -> Option<EllipsisStyleOwner> {
    (source_range.start < source_range.end)
        .then(|| {
            line.runs.iter().find(|run| {
                run.source_range.start <= source_range.start
                    && source_range.end <= run.source_range.end
            })
        })
        .flatten()
        .map(|run| EllipsisStyleOwner {
            source_range,
            kind: run.kind,
        })
}

fn ellipsis_style_owner(
    line: &CandidateLine,
    prefix_end: usize,
    suffix_start: usize,
    placement: EllipsisPlacement,
) -> Option<EllipsisStyleOwner> {
    let preceding = || {
        line.runs.iter().rev().find(|run| {
            run.source_range.start < run.source_range.end
                && run.visual_range.start < prefix_end
                && prefix_end <= run.visual_range.end
        })
    };
    let following = || {
        line.runs.iter().find(|run| {
            run.source_range.start < run.source_range.end
                && run.visual_range.start <= suffix_start
                && suffix_start < run.visual_range.end
        })
    };
    let run = match placement {
        EllipsisPlacement::Start => following().or_else(preceding),
        EllipsisPlacement::End | EllipsisPlacement::EndWord | EllipsisPlacement::Middle => {
            preceding().or_else(following)
        }
    }
    .or_else(|| {
        line.runs
            .iter()
            .find(|run| run.source_range.start < run.source_range.end)
    })?;
    Some(EllipsisStyleOwner {
        source_range: run.source_range,
        kind: run.kind,
    })
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
) -> TextLayoutOutcome<bool> {
    if line.text.is_empty() {
        return TextShapingOutcome::Ready(false);
    }
    line_text_fits_with_provider(&line.text, max_width, style, provider).map(|fits| !fits)
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

#[cfg(test)]
#[path = "tests/ellipsis.rs"]
mod tests;
