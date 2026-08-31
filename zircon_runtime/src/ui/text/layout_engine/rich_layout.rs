use crate::core::framework::text::TextLayoutError;
use crate::text::layout::{
    ELLIPSIS, RichWordWrapMode, layout_rich_text_glyph_wrapped_with_provider,
    layout_rich_text_with_provider as layout_rich_text_items_with_provider,
    layout_rich_text_word_wrapped_with_provider, measured_grapheme_widths_with_provider,
    resolve_rich_run_style, rich_forced_line_ranges, rich_glyph_line_ranges_with_provider,
    soft_hyphen_break_suffix_at,
};
use crate::text::shaping::{TextLayoutOutcome, TextShapeRunProvider, TextShapingOutcome};
use crate::text::{LayoutItem, SharedTextLayoutSession};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine, UiResolvedTextRun, UiRichTextFormat,
    UiTextDirection, UiTextRange, UiTextWrap, UiTextWritingMode,
};

use super::super::rich_text::UiParsedText;
use super::candidate_line::{CandidateLine, append_virtual_discretionary_hyphen};
use super::ellipsis::{
    ellipsis_style_owner_source_range, ellipsize_line_with_advances_and_style_owner,
    is_ellipsis_overflow,
};
use super::layout_result::LayoutWithoutArtifact;
use super::line_box::aligned_x;
use super::{virtual_fragment_sequence, visual_order};
use crate::text::text_style;

mod profile;

use profile::{RichShapeProfilePhase, RichShapeProfileProvider};

pub(super) fn layout_rich_text_with_provider(
    parsed: &UiParsedText,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    font_size: f32,
    direction: UiTextDirection,
    provider: &mut SharedTextLayoutSession,
) -> TextLayoutOutcome<Option<LayoutWithoutArtifact>> {
    if matches!(style.rich_text_format, UiRichTextFormat::Plain)
        || !matches!(style.text_writing_mode, UiTextWritingMode::HorizontalTb)
    {
        return TextShapingOutcome::Ready(None);
    }
    let neutral_style = text_style(style);

    let wrap_width = match if super::paragraph_layout::has_block_layout(parsed) {
        super::paragraph_layout::conservative_rich_width_with_provider(
            parsed,
            style,
            frame.width,
            provider,
        )
    } else {
        TextShapingOutcome::Ready(frame.width.max(0.0))
    } {
        TextShapingOutcome::Ready(width) => width,
        TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
        TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
    };
    let (rich_layout, source_ranges) = match style.wrap {
        UiTextWrap::Glyph => {
            let max_width = wrap_width;
            let ranges = match {
                crate::profile_scope!("runtime", "text.layout", "rich_range_index");
                let mut profiled =
                    RichShapeProfileProvider::new(provider, RichShapeProfilePhase::RangeIndex);
                rich_glyph_line_ranges_with_provider(
                    parsed,
                    &neutral_style,
                    max_width,
                    &mut profiled,
                )
            } {
                TextShapingOutcome::Ready(ranges) => ranges,
                TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
                TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
            };
            let layout = match {
                crate::profile_scope!("runtime", "text.layout", "rich_layout_materialization");
                let mut profiled =
                    RichShapeProfileProvider::new(provider, RichShapeProfilePhase::Layout);
                layout_rich_text_glyph_wrapped_with_provider(
                    parsed,
                    &neutral_style,
                    max_width,
                    &mut profiled,
                )
            } {
                TextShapingOutcome::Ready(layout) => layout,
                TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
                TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
            };
            (layout, ranges)
        }
        UiTextWrap::None => {
            let layout = match {
                crate::profile_scope!("runtime", "text.layout", "rich_layout_materialization");
                let mut profiled =
                    RichShapeProfileProvider::new(provider, RichShapeProfilePhase::Layout);
                layout_rich_text_items_with_provider(parsed, &neutral_style, &mut profiled)
            } {
                TextShapingOutcome::Ready(layout) => layout,
                TextShapingOutcome::Deferred(error) => {
                    return TextShapingOutcome::Deferred(error);
                }
                TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
            };
            let ranges = match rich_forced_line_ranges(parsed.text()) {
                TextShapingOutcome::Ready(ranges) => ranges,
                TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
                TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
            };
            (layout, ranges)
        }
        UiTextWrap::Word | UiTextWrap::WordSmart => {
            match {
                crate::profile_scope!("runtime", "text.layout", "rich_layout_materialization");
                let mut profiled =
                    RichShapeProfileProvider::new(provider, RichShapeProfilePhase::Layout);
                layout_rich_text_word_wrapped_with_provider(
                    parsed,
                    &neutral_style,
                    wrap_width,
                    if matches!(style.wrap, UiTextWrap::WordSmart) {
                        RichWordWrapMode::WordSmart
                    } else {
                        RichWordWrapMode::Word
                    },
                    &mut profiled,
                )
            } {
                TextShapingOutcome::Ready(layout) => layout,
                TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
                TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
            }
        }
    };
    let clip = clip_frame.unwrap_or(frame);
    let mut resolved_lines = Vec::new();
    let mut retained_virtual_line_sequences = Vec::new();
    let mut overflow_clipped =
        rich_layout.size.x > frame.width.max(0.0) || rich_layout.size.y > frame.height.max(0.0);
    let mut max_line_height = 0.0_f32;
    let mut unclipped_measured_width = 0.0_f32;
    let mut unclipped_measured_height = 0.0_f32;
    let mut previous_paragraph_start = None;
    let inline_source_ranges = parsed
        .runs
        .iter()
        .filter(|run| run.inline().is_some())
        .map(|run| run.source_range)
        .collect::<Vec<_>>();
    let paragraph_constraints =
        match super::paragraph_layout::resolve_paragraph_line_constraints_with_provider(
            parsed,
            style,
            frame.width,
            provider,
        ) {
            TextShapingOutcome::Ready(constraints) => constraints,
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        };
    {
        crate::profile_scope!("runtime", "text.layout", "ui_rich_item_projection");
        let mut profiled =
            RichShapeProfileProvider::new(provider, RichShapeProfilePhase::UiItemProjection);
        for (rich_line, source_range) in rich_layout.lines.iter().zip(source_ranges) {
            let (Ok(start), Ok(end)) = (
                usize::try_from(source_range.0),
                usize::try_from(source_range.1),
            ) else {
                return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
            };
            let source_range = UiTextRange { start, end };
            let line_height = rich_line.ascent + rich_line.descent;
            let paragraph_start = crate::text::hard_line_start(parsed.text(), source_range.start);
            let first_physical_line = previous_paragraph_start != Some(paragraph_start);
            previous_paragraph_start = Some(paragraph_start);
            let constraints =
                paragraph_constraints.for_source_offset(source_range.start, first_physical_line);
            max_line_height = max_line_height.max(line_height);
            let (Ok(item_start), Ok(item_end)) = (
                usize::try_from(rich_line.item_range.0),
                usize::try_from(rich_line.item_range.1),
            ) else {
                return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
            };
            let Some(items) = rich_layout.items.get(item_start..item_end) else {
                return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
            };
            let mut glyph_advances = Vec::new();
            for item in items {
                let advances = match item_advances(item, parsed, style, &mut profiled) {
                    TextShapingOutcome::Ready(advances) => advances,
                    TextShapingOutcome::Deferred(error) => {
                        return TextShapingOutcome::Deferred(error);
                    }
                    TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
                };
                glyph_advances.extend(advances);
            }
            let Some(line_text) = parsed.text().get(source_range.start..source_range.end) else {
                return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
            };
            let mut visual_line = CandidateLine {
                text: line_text.to_string(),
                source_range,
                runs: resolved_runs_for_line(parsed, source_range, direction),
                virtual_source_receipts: Vec::new(),
                pending_break_suffix: None,
                ellipsized: false,
            };
            match append_virtual_soft_hyphen_break_suffix(
                &mut visual_line,
                &mut glyph_advances,
                parsed,
                style,
                source_range.end,
                &mut profiled,
            ) {
                TextShapingOutcome::Ready(()) => {}
                TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
                TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
            }
            if is_ellipsis_overflow(style.text_overflow) {
                let style_owner_source_range = ellipsis_style_owner_source_range(
                    &visual_line,
                    &glyph_advances,
                    constraints.max_width,
                    style.text_overflow,
                );
                let ellipsis_style = style_owner_source_range
                    .and_then(|source_range| {
                        parsed.runs.iter().find(|run| {
                            run.source_range.start <= source_range.start
                                && source_range.end <= run.source_range.end
                        })
                    })
                    .map_or_else(
                        || neutral_style.clone(),
                        |run| resolve_rich_run_style(&neutral_style, run.style()),
                    );
                let ellipsis_advance = match measured_grapheme_widths_with_provider(
                    ELLIPSIS,
                    &ellipsis_style,
                    &mut profiled,
                ) {
                    TextShapingOutcome::Ready(advances) => {
                        advances.into_iter().next().unwrap_or_default()
                    }
                    TextShapingOutcome::Deferred(error) => {
                        return TextShapingOutcome::Deferred(error);
                    }
                    TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
                };
                ellipsize_line_with_advances_and_style_owner(
                    &mut visual_line,
                    &mut glyph_advances,
                    constraints.max_width,
                    ellipsis_advance,
                    style.text_overflow,
                    style_owner_source_range,
                );
                overflow_clipped |= visual_line.ellipsized;
            }
            let mut virtual_sequence =
                virtual_fragment_sequence::capture_with_external_source_ranges(
                    &visual_line,
                    direction,
                    &inline_source_ranges,
                );
            let visual_order = if let Some(sequence) = virtual_sequence.as_mut() {
                visual_order::apply_visual_order_with_virtual_sequence(
                    &mut visual_line,
                    direction,
                    Some(sequence),
                    Some(&mut glyph_advances),
                )
            } else {
                visual_order::apply_visual_order_with_advances(
                    &mut visual_line,
                    parsed.text(),
                    direction,
                    &mut glyph_advances,
                )
            };
            match visual_order {
                Ok(()) => {}
                Err(_) => return TextShapingOutcome::failed(TextLayoutError::BidiInvariant),
            }
            let measured_width = glyph_advances.iter().copied().sum::<f32>();
            let resolved_source_range = visual_line.source_range;
            let line_width = measured_width.min(constraints.max_width);
            let line_align = constraints.align;
            let content_frame =
                super::paragraph_layout::inset_logical_start(frame, constraints.inset, direction);
            let placement_frame = UiFrame::new(
                content_frame.x,
                frame.y + rich_line.origin.y,
                content_frame.width,
                line_height,
            );
            let line_frame = UiFrame::new(
                aligned_x(content_frame, line_width, line_align, direction),
                frame.y + rich_line.origin.y,
                measured_width,
                line_height,
            );
            unclipped_measured_width = unclipped_measured_width.max(measured_width);
            unclipped_measured_height =
                unclipped_measured_height.max(line_frame.bottom() - frame.y);
            if placement_frame.intersection(clip).is_none() {
                overflow_clipped = true;
                continue;
            }
            retained_virtual_line_sequences.push(virtual_sequence);
            resolved_lines.push(UiResolvedTextLine {
                text: visual_line.text,
                frame: line_frame,
                placement_frame,
                source_range: resolved_source_range,
                visual_range: UiTextRange {
                    start: 0,
                    end: visual_line
                        .runs
                        .last()
                        .map(|run| run.visual_range.end)
                        .unwrap_or_default(),
                },
                measured_width,
                glyph_advances,
                baseline: rich_line.baseline,
                direction,
                runs: visual_line.runs,
                ellipsized: visual_line.ellipsized,
            });
        }
    }

    let layout = UiResolvedTextLayout {
        text_align: style.text_align,
        wrap: style.wrap,
        direction,
        writing_mode: style.text_writing_mode,
        overflow: style.text_overflow,
        font_size,
        line_height: max_line_height,
        measured_width: unclipped_measured_width,
        measured_height: unclipped_measured_height,
        source_range: UiTextRange {
            start: 0,
            end: parsed.text().len(),
        },
        lines: resolved_lines,
        boxes: Vec::new(),
        overflow_clipped,
        editable: None,
        rich_text_artifact: None,
    };
    TextShapingOutcome::Ready(Some(LayoutWithoutArtifact::with_virtual_line_sequences(
        layout,
        retained_virtual_line_sequences,
    )))
}

fn append_virtual_soft_hyphen_break_suffix<P>(
    line: &mut CandidateLine,
    glyph_advances: &mut Vec<f32>,
    parsed: &UiParsedText,
    base_style: &UiResolvedStyle,
    break_end: usize,
    provider: &mut P,
) -> TextLayoutOutcome<()>
where
    P: TextShapeRunProvider + ?Sized,
{
    append_soft_hyphen_break_suffix_with_projection(
        line,
        glyph_advances,
        parsed,
        base_style,
        break_end,
        provider,
    )
}

pub(super) fn append_soft_hyphen_break_suffix<P>(
    line: &mut CandidateLine,
    glyph_advances: &mut Vec<f32>,
    parsed: &UiParsedText,
    base_style: &UiResolvedStyle,
    break_end: usize,
    provider: &mut P,
) -> TextLayoutOutcome<()>
where
    P: TextShapeRunProvider + ?Sized,
{
    append_soft_hyphen_break_suffix_with_projection(
        line,
        glyph_advances,
        parsed,
        base_style,
        break_end,
        provider,
    )
}

fn append_soft_hyphen_break_suffix_with_projection<P>(
    line: &mut CandidateLine,
    glyph_advances: &mut Vec<f32>,
    parsed: &UiParsedText,
    base_style: &UiResolvedStyle,
    break_end: usize,
    provider: &mut P,
) -> TextLayoutOutcome<()>
where
    P: TextShapeRunProvider + ?Sized,
{
    let Some(suffix) = soft_hyphen_break_suffix_at(parsed.text(), break_end) else {
        return TextShapingOutcome::Ready(());
    };
    let consumed = suffix.consumed_source_range();
    let source_range = UiTextRange {
        start: consumed.start,
        end: consumed.end,
    };
    let run = parsed.runs.iter().find(|run| {
        run.source_range.start <= source_range.start && source_range.end <= run.source_range.end
    });
    let kind = run.map_or(
        zircon_runtime_interface::ui::surface::UiTextRunKind::Plain,
        |run| run.kind,
    );
    let suffix_style = run.map_or_else(
        || text_style(base_style),
        |run| resolve_rich_run_style(&text_style(base_style), run.style()),
    );
    let suffix_advance =
        match measured_grapheme_widths_with_provider(suffix.marker_text(), &suffix_style, provider)
        {
            TextShapingOutcome::Ready(advances) => advances.into_iter().next().unwrap_or_default(),
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        };
    append_virtual_discretionary_hyphen(line, kind, suffix);
    glyph_advances.push(suffix_advance);
    TextShapingOutcome::Ready(())
}

pub(super) fn resolved_runs_for_line(
    parsed: &UiParsedText,
    line_range: UiTextRange,
    direction: UiTextDirection,
) -> Vec<UiResolvedTextRun> {
    parsed
        .runs
        .iter()
        .filter_map(|run| {
            let start = run.source_range.start.max(line_range.start);
            let end = run.source_range.end.min(line_range.end);
            let text = parsed.text().get(start..end)?.to_string();
            (start < end).then_some(UiResolvedTextRun {
                kind: run.kind,
                text,
                source_range: UiTextRange { start, end },
                visual_range: UiTextRange {
                    start: start - line_range.start,
                    end: end - line_range.start,
                },
                direction,
            })
        })
        .collect()
}

fn item_advances<P>(
    item: &LayoutItem,
    parsed: &UiParsedText,
    base_style: &UiResolvedStyle,
    provider: &mut P,
) -> TextLayoutOutcome<Vec<f32>>
where
    P: TextShapeRunProvider + ?Sized,
{
    match item {
        LayoutItem::Inline { advance, .. } => vec![*advance],
        LayoutItem::Text {
            run_index,
            source_range,
            ..
        } => {
            let Some(run) = parsed.rich.parsed().runs.get(*run_index as usize) else {
                return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
            };
            let Ok(start) = usize::try_from(source_range.0) else {
                return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
            };
            let Ok(end) = usize::try_from(source_range.1) else {
                return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
            };
            let Some(text) = parsed.text().get(start..end) else {
                return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
            };
            let style = resolve_rich_run_style(&text_style(base_style), &run.style);
            measured_grapheme_widths_with_provider(text, &style, provider)
        }
    }
}
