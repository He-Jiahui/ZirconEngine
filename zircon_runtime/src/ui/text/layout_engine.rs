use crate::graphics::text::layout::{
    line_metrics_with_provider, measure_text_size as measure_backend_text_size,
    measure_text_size_with_provider as measure_backend_text_size_with_provider,
    measure_text_source_range_width as measure_backend_text_source_range_width, TextLineMetrics,
};
use crate::graphics::text::shaping::{DirectTextShapeRunProvider, TextShapeRunProvider};
use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine, UiTextAlign, UiTextOverflow,
    UiTextRange, UiTextWritingMode,
};

use crate::core::framework::render::ParagraphOverride;

use super::rich_text::parse_source_text;

mod candidate_line;
mod direction;
mod ellipsis;
mod line_box;
mod overflow_style;
mod paragraph_layout;
mod range_mapping;
mod rich_inline;
mod rich_inline_vertical;
mod rich_table;
mod vertical;
mod visual_order;
mod wrapping;

use ellipsis::{
    ellipsize_line_with_provider, is_ellipsis_overflow, line_overflows_horizontally_with_provider,
    merge_clipped_lines_for_tail_preserving_ellipsis,
};
use line_box::{aligned_x, resolve_line_widths_with_provider, text_advance, MIN_TEXT_FONT_SIZE};
use wrapping::wrap_source_runs_with_provider;

pub(crate) use direction::resolve_direction as resolve_text_direction;

pub(crate) fn measure_text_size(text: &str, style: &UiResolvedStyle) -> UiSize {
    let parsed = parse_source_text(text, style.rich_text_format);
    measure_backend_text_size(&parsed.text, style)
}

pub(crate) fn measure_text_size_with_provider<P>(
    text: &str,
    style: &UiResolvedStyle,
    provider: &mut P,
) -> UiSize
where
    P: TextShapeRunProvider + ?Sized,
{
    let parsed = parse_source_text(text, style.rich_text_format);
    measure_backend_text_size_with_provider(&parsed.text, style, provider)
}

pub(crate) fn measure_text_source_range_width(
    text: &str,
    style: &UiResolvedStyle,
    range: UiTextRange,
) -> f32 {
    let parsed = parse_source_text(text, style.rich_text_format);
    measure_backend_text_source_range_width(&parsed.text, style, range)
}

pub(crate) fn layout_text(
    text: &str,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
) -> UiResolvedTextLayout {
    let mut provider = DirectTextShapeRunProvider;
    layout_text_with_provider(text, style, frame, clip_frame, &mut provider)
}

pub(crate) fn layout_text_with_provider<P>(
    text: &str,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    provider: &mut P,
) -> UiResolvedTextLayout
where
    P: TextShapeRunProvider + ?Sized,
{
    let parsed = parse_source_text(text, style.rich_text_format);
    layout_parsed_text_with_provider(&parsed, style, frame, clip_frame, provider)
}

pub(super) fn layout_parsed_text_with_provider<P>(
    parsed: &super::rich_text::UiParsedText,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    provider: &mut P,
) -> UiResolvedTextLayout
where
    P: TextShapeRunProvider + ?Sized,
{
    if let Some(layout) =
        rich_table::layout_rich_tables_with_provider(parsed, style, frame, clip_frame, provider)
    {
        return layout;
    }
    layout_parsed_text_without_tables_with_provider(parsed, style, frame, clip_frame, provider)
}

pub(super) fn layout_parsed_text_without_tables_with_provider<P>(
    parsed: &super::rich_text::UiParsedText,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    provider: &mut P,
) -> UiResolvedTextLayout
where
    P: TextShapeRunProvider + ?Sized,
{
    let visible_text = parsed.text.as_str();
    let effective_style =
        resolve_overflow_style_with_provider(visible_text, style, frame, provider);
    let style = &effective_style;
    let font_size = style.font_size.max(MIN_TEXT_FONT_SIZE);
    let metrics: TextLineMetrics = line_metrics_with_provider(style, provider);
    let line_height = metrics.line_height;
    if matches!(style.text_writing_mode, UiTextWritingMode::VerticalRl) {
        return vertical::layout_vertical_text_with_provider(
            &parsed, style, frame, clip_frame, font_size, metrics, provider,
        );
    }

    let direction = resolve_text_direction(visible_text, style.text_direction);
    if let Some(layout) = rich_inline::layout_inline_rich_text_with_provider(
        &parsed, style, frame, clip_frame, font_size, direction, provider,
    ) {
        return layout;
    }
    let source_runs = &parsed.runs;
    let max_width = frame.width.max(text_advance(font_size));
    let block_layout = paragraph_layout::has_block_layout(&parsed);
    let mut lines = if block_layout {
        paragraph_layout::wrap_block_paragraphs_with_provider(&parsed, style, frame.width, provider)
    } else {
        wrap_source_runs_with_provider(source_runs, style.wrap, max_width, style, provider)
    };
    let clip = clip_frame.unwrap_or(frame);
    let line_capacity = (frame.height.max(line_height) / line_height)
        .floor()
        .max(1.0) as usize;
    let mut overflow_clipped = lines.len() > line_capacity;
    if is_ellipsis_overflow(style.text_overflow) && overflow_clipped {
        if matches!(
            style.text_overflow,
            UiTextOverflow::EllipsisStart | UiTextOverflow::EllipsisMiddle
        ) {
            merge_clipped_lines_for_tail_preserving_ellipsis(&mut lines, line_capacity);
        }
        lines.truncate(line_capacity);
        let last_index = lines.len().saturating_sub(1);
        let available_width =
            block_line_constraints(&parsed, style, frame.width, &lines, last_index, provider)
                .max_width;
        if let Some(last) = lines.last_mut() {
            ellipsize_line_with_provider(
                last,
                available_width,
                style,
                style.text_overflow,
                provider,
            );
        }
    }
    if is_ellipsis_overflow(style.text_overflow) {
        for index in 0..lines.len() {
            let available_width =
                block_line_constraints(&parsed, style, frame.width, &lines, index, provider)
                    .max_width;
            let line = &mut lines[index];
            if !line.ellipsized
                && line_overflows_horizontally_with_provider(line, available_width, style, provider)
            {
                ellipsize_line_with_provider(
                    line,
                    available_width,
                    style,
                    style.text_overflow,
                    provider,
                );
                overflow_clipped = true;
            }
        }
    }
    for line in &mut lines {
        visual_order::apply_visual_order(line, visible_text, direction);
    }

    let mut resolved_lines = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let y = frame.y + index as f32 * line_height;
        let is_last_line = index + 1 == lines.len();
        let constraints =
            block_line_constraints(&parsed, style, frame.width, &lines, index, provider);
        let line_align = constraints.align;
        let mut line_style = style.clone();
        line_style.text_align = line_align;
        let (measured_width, glyph_advances, line_width) = resolve_line_widths_with_provider(
            line,
            &line_style,
            constraints.max_width,
            is_last_line,
            provider,
        );
        let content_frame =
            paragraph_layout::inset_logical_start(frame, constraints.inset, direction);
        let line_frame = UiFrame::new(
            aligned_x(content_frame, line_width, line_align, direction),
            y,
            line_width,
            line_height,
        );
        if line_frame.intersection(clip).is_some() {
            resolved_lines.push(UiResolvedTextLine {
                text: line.text.clone(),
                frame: line_frame,
                source_range: line.source_range,
                visual_range: UiTextRange {
                    start: 0,
                    end: line.text.len(),
                },
                measured_width,
                glyph_advances,
                baseline: metrics.baseline,
                direction,
                runs: line.runs.clone(),
                ellipsized: line.ellipsized,
            });
        } else {
            overflow_clipped = true;
        }
    }

    let measured_width = resolved_lines
        .iter()
        .map(|line| line.measured_width)
        .fold(0.0_f32, f32::max);
    let measured_height = resolved_lines.len() as f32 * line_height;
    UiResolvedTextLayout {
        text_align: style.text_align,
        wrap: style.wrap,
        direction,
        writing_mode: style.text_writing_mode,
        overflow: style.text_overflow,
        font_size,
        line_height,
        measured_width,
        measured_height,
        source_range: UiTextRange {
            start: 0,
            end: visible_text.len(),
        },
        lines: resolved_lines,
        boxes: Vec::new(),
        overflow_clipped,
        editable: None,
    }
}

fn block_line_constraints<P>(
    parsed: &super::rich_text::UiParsedText,
    style: &UiResolvedStyle,
    frame_width: f32,
    lines: &[candidate_line::CandidateLine],
    index: usize,
    provider: &mut P,
) -> paragraph_layout::LineConstraints
where
    P: TextShapeRunProvider + ?Sized,
{
    let line = &lines[index];
    let paragraph_start =
        paragraph_layout::physical_paragraph_start(&parsed.text, line.source_range.start);
    let first_physical_line = index == 0
        || paragraph_layout::physical_paragraph_start(
            &parsed.text,
            lines[index - 1].source_range.start,
        ) != paragraph_start;
    paragraph_layout::line_constraints_with_provider(
        parsed,
        style,
        frame_width,
        line.source_range.start,
        first_physical_line,
        provider,
    )
}

fn paragraph_align_for_offset(
    paragraphs: &[((u32, u32), ParagraphOverride)],
    offset: usize,
    fallback: UiTextAlign,
) -> UiTextAlign {
    let offset = u32::try_from(offset).unwrap_or(u32::MAX);
    paragraphs
        .iter()
        .find(|(range, _)| range.0 <= offset && offset < range.1)
        .and_then(|(_, paragraph)| paragraph.align)
        .unwrap_or(fallback)
}

fn resolve_overflow_style_with_provider<P>(
    text: &str,
    style: &UiResolvedStyle,
    frame: UiFrame,
    provider: &mut P,
) -> UiResolvedStyle
where
    P: TextShapeRunProvider + ?Sized,
{
    let max_extent = if matches!(style.text_writing_mode, UiTextWritingMode::VerticalRl) {
        frame.height
    } else {
        frame.width
    };
    overflow_style::resolve(text, style, max_extent, |text, style| {
        measure_backend_text_size_with_provider(text, style, provider)
    })
}

#[cfg(test)]
mod tests;
