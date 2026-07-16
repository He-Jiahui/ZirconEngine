use crate::text::layout::{
    layout_rich_text_glyph_wrapped_with_provider, layout_rich_text_with_provider,
    layout_rich_text_word_wrapped_with_provider, measured_grapheme_widths_with_provider,
    resolve_rich_run_style, rich_forced_line_ranges, rich_glyph_line_ranges_with_provider,
    RichWordWrapMode, ELLIPSIS,
};
use crate::text::LayoutItem;
use crate::text::SharedTextLayoutSession;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine, UiResolvedTextRun, UiTextDirection,
    UiTextRange, UiTextWrap, UiTextWritingMode,
};

use super::super::adapter::text_style;
use super::super::rich_text::UiParsedText;
use super::candidate_line::CandidateLine;
use super::ellipsis::{ellipsize_line_with_advances, is_ellipsis_overflow};
use super::line_box::aligned_x;
use super::visual_order::apply_visual_order_with_advances;

pub(super) fn layout_inline_rich_text_with_provider(
    parsed: &UiParsedText,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    font_size: f32,
    direction: UiTextDirection,
    provider: &mut SharedTextLayoutSession,
) -> Option<UiResolvedTextLayout> {
    if !parsed.runs.iter().any(|run| run.inline.is_some())
        || !matches!(style.text_writing_mode, UiTextWritingMode::HorizontalTb)
    {
        return None;
    }
    let neutral_style = text_style(style);

    let wrap_width = if super::paragraph_layout::has_block_layout(parsed) {
        super::paragraph_layout::conservative_rich_width_with_provider(
            parsed,
            style,
            frame.width,
            provider,
        )
    } else {
        frame.width.max(0.0)
    };
    let (rich_layout, source_ranges) = match style.wrap {
        UiTextWrap::Glyph => {
            let max_width = wrap_width;
            let ranges = rich_glyph_line_ranges_with_provider(
                &parsed.rich,
                &neutral_style,
                max_width,
                provider,
            );
            (
                layout_rich_text_glyph_wrapped_with_provider(
                    &parsed.rich,
                    &neutral_style,
                    max_width,
                    provider,
                ),
                ranges,
            )
        }
        UiTextWrap::None => (
            layout_rich_text_with_provider(&parsed.rich, &neutral_style, provider),
            rich_forced_line_ranges(&parsed.text),
        ),
        UiTextWrap::Word | UiTextWrap::WordSmart => layout_rich_text_word_wrapped_with_provider(
            &parsed.rich,
            &neutral_style,
            wrap_width,
            if matches!(style.wrap, UiTextWrap::WordSmart) {
                RichWordWrapMode::WordSmart
            } else {
                RichWordWrapMode::Word
            },
            provider,
        ),
    };
    let clip = clip_frame.unwrap_or(frame);
    let mut resolved_lines = Vec::new();
    let mut overflow_clipped =
        rich_layout.size.x > frame.width.max(0.0) || rich_layout.size.y > frame.height.max(0.0);
    let mut max_line_height = 0.0_f32;
    let mut previous_paragraph_start = None;
    for (rich_line, source_range) in rich_layout.lines.iter().zip(source_ranges) {
        let source_range = UiTextRange {
            start: usize::try_from(source_range.0).ok()?,
            end: usize::try_from(source_range.1).ok()?,
        };
        let line_height = rich_line.ascent + rich_line.descent;
        let paragraph_start =
            super::paragraph_layout::physical_paragraph_start(&parsed.text, source_range.start);
        let first_physical_line = previous_paragraph_start != Some(paragraph_start);
        previous_paragraph_start = Some(paragraph_start);
        let constraints = super::paragraph_layout::line_constraints_with_provider(
            parsed,
            style,
            frame.width,
            source_range.start,
            first_physical_line,
            provider,
        );
        max_line_height = max_line_height.max(line_height);
        let item_start = usize::try_from(rich_line.item_range.0).ok()?;
        let item_end = usize::try_from(rich_line.item_range.1).ok()?;
        let mut glyph_advances = rich_layout
            .items
            .get(item_start..item_end)?
            .iter()
            .flat_map(|item| item_advances(item, parsed, style, provider))
            .collect::<Vec<_>>();
        let line_text = parsed
            .text
            .get(source_range.start..source_range.end)?
            .to_string();
        let mut visual_line = CandidateLine {
            text: line_text,
            source_range,
            runs: resolved_runs_for_line(parsed, source_range, direction),
            pending_break_suffix: None,
            ellipsized: false,
        };
        apply_visual_order_with_advances(
            &mut visual_line,
            &parsed.text,
            direction,
            &mut glyph_advances,
        );
        if is_ellipsis_overflow(style.text_overflow) {
            let ellipsis_advance =
                measured_grapheme_widths_with_provider(ELLIPSIS, &neutral_style, provider)
                    .into_iter()
                    .next()
                    .unwrap_or_default();
            ellipsize_line_with_advances(
                &mut visual_line,
                &mut glyph_advances,
                constraints.max_width,
                ellipsis_advance,
                style.text_overflow,
            );
            overflow_clipped |= visual_line.ellipsized;
        }
        let measured_width = glyph_advances.iter().copied().sum::<f32>();
        let line_width = measured_width.min(constraints.max_width);
        let line_align = constraints.align;
        let content_frame =
            super::paragraph_layout::inset_logical_start(frame, constraints.inset, direction);
        let line_frame = UiFrame::new(
            aligned_x(content_frame, line_width, line_align, direction),
            frame.y + rich_line.origin.y,
            line_width,
            line_height,
        );
        if line_frame.intersection(clip).is_none() {
            overflow_clipped = true;
            continue;
        }
        resolved_lines.push(UiResolvedTextLine {
            text: visual_line.text,
            frame: line_frame,
            source_range,
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

    let measured_width = resolved_lines
        .iter()
        .map(|line| line.measured_width)
        .fold(0.0_f32, f32::max);
    let measured_height = resolved_lines
        .iter()
        .map(|line| line.frame.bottom() - frame.y)
        .fold(0.0_f32, f32::max);
    Some(UiResolvedTextLayout {
        text_align: style.text_align,
        wrap: style.wrap,
        direction,
        writing_mode: style.text_writing_mode,
        overflow: style.text_overflow,
        font_size,
        line_height: max_line_height,
        measured_width,
        measured_height,
        source_range: UiTextRange {
            start: 0,
            end: parsed.text.len(),
        },
        lines: resolved_lines,
        boxes: Vec::new(),
        overflow_clipped,
        editable: None,
    })
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
            let text = parsed.text.get(start..end)?.to_string();
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

fn item_advances(
    item: &LayoutItem,
    parsed: &UiParsedText,
    base_style: &UiResolvedStyle,
    provider: &mut SharedTextLayoutSession,
) -> Vec<f32> {
    match item {
        LayoutItem::Inline { advance, .. } => vec![*advance],
        LayoutItem::Text {
            run_index,
            source_range,
            ..
        } => {
            let Some(run) = parsed.rich.runs.get(*run_index as usize) else {
                return Vec::new();
            };
            let Ok(start) = usize::try_from(source_range.0) else {
                return Vec::new();
            };
            let Ok(end) = usize::try_from(source_range.1) else {
                return Vec::new();
            };
            let Some(text) = parsed.text.get(start..end) else {
                return Vec::new();
            };
            let style = resolve_rich_run_style(&text_style(base_style), &run.style);
            measured_grapheme_widths_with_provider(text, &style, provider)
        }
    }
}
