use crate::text::layout::{
    line_metrics_with_provider,
    measure_text_size_with_provider as measure_backend_text_size_with_provider,
    measure_text_source_range_width_with_provider as measure_backend_text_source_range_width_with_provider,
    TextLineMetrics,
};
use crate::text::{
    build_resolved_text_glyph_artifact, build_resolved_text_glyph_artifact_with_shared_source,
    register_resolved_text_glyph_artifact, text_style, SharedTextLayoutSession, TextDocumentKey,
};
use std::sync::Arc;
use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine, UiTextAlign, UiTextOverflow,
    UiTextRange, UiTextWrap, UiTextWritingMode,
};

use crate::text::register_compiled_rich_text_artifact;

use super::rich_text::{parse_source_text, UiParsedText};

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
mod viewport;
mod visual_order;
mod wrapping;

use super::resolved_layout::UiTextViewport;
use ellipsis::{
    ellipsize_line_with_provider, is_ellipsis_overflow, line_overflows_horizontally_with_provider,
    merge_clipped_lines_for_tail_preserving_ellipsis,
};
use line_box::{
    aligned_x, available_wrap_extent, materialize_arabic_tatweels_for_justified_line,
    resolve_line_widths_with_provider, MIN_TEXT_FONT_SIZE,
};
use viewport::visible_plain_text_lines;
use wrapping::wrap_source_runs_with_provider;

pub(crate) use direction::resolve_direction as resolve_text_direction;

pub(crate) fn measure_text_size(text: &str, style: &UiResolvedStyle) -> UiSize {
    let mut session = SharedTextLayoutSession::new();
    measure_text_size_with_provider(text, style, &mut session)
}

pub(crate) fn measure_text_size_with_provider(
    text: &str,
    style: &UiResolvedStyle,
    provider: &mut SharedTextLayoutSession,
) -> UiSize {
    let parsed = parse_source_text(text, style.rich_text_format.into());
    if !matches!(
        style.rich_text_format,
        zircon_runtime_interface::ui::surface::UiRichTextFormat::Plain
    ) || matches!(style.text_writing_mode, UiTextWritingMode::VerticalRl)
    {
        let mut intrinsic_style = style.clone();
        intrinsic_style.wrap = UiTextWrap::None;
        intrinsic_style.text_overflow = UiTextOverflow::Clip;
        let em = intrinsic_style
            .line_height
            .max(intrinsic_style.font_size)
            .max(1.0);
        let extent = (parsed.text().len().max(1) as f32)
            .mul_add(em, em)
            .min(f32::MAX.sqrt());
        let layout = layout_parsed_text_with_provider(
            &parsed,
            &intrinsic_style,
            UiFrame::new(0.0, 0.0, extent, extent),
            None,
            provider,
        );
        return UiSize::new(layout.measured_width, layout.measured_height);
    }
    measure_backend_text_size_with_provider(parsed.text(), &text_style(style), provider).into()
}

pub(crate) fn measure_unwrapped_text_height(text: &str, style: &UiResolvedStyle) -> Option<f32> {
    let mut session = SharedTextLayoutSession::new();
    measure_unwrapped_text_height_with_provider(text, style, &mut session)
}

pub(crate) fn measure_unwrapped_text_height_with_provider(
    text: &str,
    style: &UiResolvedStyle,
    provider: &mut SharedTextLayoutSession,
) -> Option<f32> {
    if text.is_empty()
        || !matches!(
            style.rich_text_format,
            zircon_runtime_interface::ui::surface::UiRichTextFormat::Plain
        )
        || !matches!(style.wrap, UiTextWrap::None)
        || !matches!(style.text_overflow, UiTextOverflow::Clip)
        || matches!(style.text_writing_mode, UiTextWritingMode::VerticalRl)
    {
        return None;
    }

    let line_height = line_metrics_with_provider(&text_style(style), provider).line_height;
    let line_count = provider
        .hard_line_count_and_window(text, None, 0..0)
        .0
        .max(1) as f32;
    Some(line_height * line_count)
}

/// Uses the same hard-line window decision as layout before a retained owner chooses whether a
/// full-document prewarm would be wasted. The line-metrics sample and hard-line index are then
/// reused by the subsequent layout request.
pub(super) fn viewport_selects_partial_plain_text(
    parsed: &UiParsedText,
    style: &UiResolvedStyle,
    viewport: UiTextViewport,
    document_key: Option<TextDocumentKey>,
    provider: &mut SharedTextLayoutSession,
) -> bool {
    let line_height = line_metrics_with_provider(&text_style(style), provider).line_height;
    visible_plain_text_lines(parsed, style, viewport, line_height, document_key, provider).is_some()
}

pub(crate) fn measure_text_source_range_width(
    text: &str,
    style: &UiResolvedStyle,
    range: UiTextRange,
) -> f32 {
    let parsed = parse_source_text(text, style.rich_text_format.into());
    let mut session = SharedTextLayoutSession::new();
    measure_backend_text_source_range_width_with_provider(
        parsed.text(),
        &text_style(style),
        range.into(),
        &mut session,
    )
}

pub(crate) fn layout_text(
    text: &str,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
) -> UiResolvedTextLayout {
    let mut provider = SharedTextLayoutSession::new();
    layout_text_with_provider(text, style, frame, clip_frame, &mut provider)
}

pub(crate) fn layout_text_with_provider(
    text: &str,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    provider: &mut SharedTextLayoutSession,
) -> UiResolvedTextLayout {
    layout_text_with_provider_and_optional_viewport(
        text, style, frame, clip_frame, None, None, provider,
    )
}

pub(crate) fn layout_text_with_viewport(
    text: &str,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    viewport: UiTextViewport,
) -> UiResolvedTextLayout {
    let mut provider = SharedTextLayoutSession::new();
    layout_text_with_provider_and_optional_viewport(
        text,
        style,
        frame,
        clip_frame,
        Some(viewport),
        None,
        &mut provider,
    )
}

pub(crate) fn layout_text_with_provider_and_viewport(
    text: &str,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    viewport: UiTextViewport,
    document_key: Option<TextDocumentKey>,
    provider: &mut SharedTextLayoutSession,
) -> UiResolvedTextLayout {
    layout_text_with_provider_and_optional_viewport(
        text,
        style,
        frame,
        clip_frame,
        Some(viewport),
        document_key,
        provider,
    )
}

fn layout_text_with_provider_and_optional_viewport(
    text: &str,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    viewport: Option<UiTextViewport>,
    document_key: Option<TextDocumentKey>,
    provider: &mut SharedTextLayoutSession,
) -> UiResolvedTextLayout {
    let parsed = parse_source_text(text, style.rich_text_format.into());
    layout_parsed_text_with_provider_and_viewport(
        &parsed,
        style,
        frame,
        clip_frame,
        viewport,
        document_key,
        provider,
    )
}

pub(super) fn layout_parsed_text_with_provider_and_viewport(
    parsed: &super::rich_text::UiParsedText,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    viewport: Option<UiTextViewport>,
    document_key: Option<TextDocumentKey>,
    provider: &mut SharedTextLayoutSession,
) -> UiResolvedTextLayout {
    let collect_profile_metrics = layout_profile_metrics_enabled();
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    let cache_report_before = collect_profile_metrics.then(|| provider.cache_report());
    let mut layout = {
        crate::profile_scope!("runtime", "text.layout", "resolve_without_artifact");
        layout_parsed_text_without_artifacts_with_viewport(
            parsed,
            style,
            frame,
            clip_frame,
            viewport,
            document_key,
            provider,
        )
    };
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    if collect_profile_metrics {
        let cache_report_after = provider.cache_report();
        let cache_report_before = cache_report_before
            .expect("active layout profiling must capture the shaped-cache baseline");
        crate::profile_counter!(
            "runtime",
            "layout_pre_artifact_shaped_cache_hit_count",
            cache_report_after
                .hit_count
                .saturating_sub(cache_report_before.hit_count)
        );
        crate::profile_counter!(
            "runtime",
            "layout_pre_artifact_shaped_cache_miss_count",
            cache_report_after
                .miss_count
                .saturating_sub(cache_report_before.miss_count)
        );
    }
    if !matches!(
        style.rich_text_format,
        zircon_runtime_interface::ui::surface::UiRichTextFormat::Plain
    ) {
        layout.rich_text_artifact = Some(register_compiled_rich_text_artifact(Arc::clone(
            &parsed.rich,
        )));
    } else if let Some(artifact) = if viewport.is_some() && parsed.source_offset() == 0 {
        build_resolved_text_glyph_artifact_with_shared_source(
            parsed.rich.shared_text(),
            style,
            &layout,
            provider,
        )
    } else {
        build_resolved_text_glyph_artifact(parsed.text(), style, &layout, provider)
    } {
        layout.rich_text_artifact = Some(register_resolved_text_glyph_artifact(Arc::new(artifact)));
    }
    layout
}

/// Tracy streams counters continuously, while the CPU recorder should remain inert when idle.
fn layout_profile_metrics_enabled() -> bool {
    #[cfg(feature = "profiling-tracy")]
    {
        return true;
    }
    #[cfg(all(feature = "profiling", not(feature = "profiling-tracy")))]
    {
        return crate::core::diagnostics::profiling::capture_active();
    }
    #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
    {
        false
    }
}

pub(super) fn layout_parsed_text_with_provider(
    parsed: &super::rich_text::UiParsedText,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    provider: &mut SharedTextLayoutSession,
) -> UiResolvedTextLayout {
    layout_parsed_text_with_provider_and_viewport(
        parsed, style, frame, clip_frame, None, None, provider,
    )
}

fn layout_parsed_text_without_artifacts_with_viewport(
    parsed: &super::rich_text::UiParsedText,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    viewport: Option<UiTextViewport>,
    document_key: Option<TextDocumentKey>,
    provider: &mut SharedTextLayoutSession,
) -> UiResolvedTextLayout {
    if let Some(layout) =
        rich_table::layout_rich_tables_with_provider(parsed, style, frame, clip_frame, provider)
    {
        return layout;
    }
    layout_parsed_text_without_tables_with_viewport(
        parsed,
        style,
        frame,
        clip_frame,
        viewport,
        document_key,
        provider,
    )
}

pub(super) fn layout_parsed_text_without_tables_with_provider(
    parsed: &super::rich_text::UiParsedText,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    provider: &mut SharedTextLayoutSession,
) -> UiResolvedTextLayout {
    layout_parsed_text_without_tables_with_viewport(
        parsed, style, frame, clip_frame, None, None, provider,
    )
}

fn layout_parsed_text_without_tables_with_viewport(
    parsed: &super::rich_text::UiParsedText,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    viewport: Option<UiTextViewport>,
    document_key: Option<TextDocumentKey>,
    provider: &mut SharedTextLayoutSession,
) -> UiResolvedTextLayout {
    let visible_text = parsed.text();
    let effective_style =
        resolve_overflow_style_with_provider(visible_text, style, frame, provider);
    let style = &effective_style;
    let font_size = style.font_size.max(MIN_TEXT_FONT_SIZE);
    let metrics: TextLineMetrics = line_metrics_with_provider(&text_style(style), provider);
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
    let max_width = available_wrap_extent(frame.width);
    let block_layout = paragraph_layout::has_block_layout(&parsed);
    let (mut lines, line_index_offset, total_line_count, virtualized) = viewport
        .and_then(|viewport| {
            visible_plain_text_lines(
                &parsed,
                style,
                viewport,
                line_height,
                document_key,
                provider,
            )
        })
        .map(|window| {
            (
                window.lines,
                window.first_line,
                window.total_line_count,
                true,
            )
        })
        .unwrap_or_else(|| {
            let lines = if block_layout {
                paragraph_layout::wrap_block_paragraphs_with_provider(
                    &parsed, style, max_width, provider,
                )
            } else {
                wrap_source_runs_with_provider(source_runs, style.wrap, max_width, style, provider)
            };
            let total_line_count = lines.len();
            (lines, 0, total_line_count, false)
        });
    let line_constraints = if block_layout {
        paragraph_layout::resolve_paragraph_line_constraints_with_provider(
            &parsed,
            style,
            frame.width,
            provider,
        )
        .for_candidates(&lines)
    } else {
        vec![
            paragraph_layout::LineConstraints {
                inset: 0.0,
                max_width: frame.width.max(0.0),
                align: style.text_align,
            };
            lines.len()
        ]
    };
    let clip = clip_frame.unwrap_or(frame);
    let line_capacity = (frame.height.max(line_height) / line_height)
        .floor()
        .max(1.0) as usize;
    let mut overflow_clipped = total_line_count > line_capacity;
    if is_ellipsis_overflow(style.text_overflow) && overflow_clipped {
        if matches!(
            style.text_overflow,
            UiTextOverflow::EllipsisStart | UiTextOverflow::EllipsisMiddle
        ) {
            merge_clipped_lines_for_tail_preserving_ellipsis(&mut lines, line_capacity);
        }
        lines.truncate(line_capacity);
        let last_index = lines.len().saturating_sub(1);
        let available_width = line_constraints[last_index].max_width;
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
            let available_width = line_constraints[index].max_width;
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
    for index in 0..lines.len() {
        let line_index = line_index_offset.saturating_add(index);
        let is_last_line = line_index.saturating_add(1) == total_line_count;
        let constraints = line_constraints[index];
        let mut line_style = style.clone();
        line_style.text_align = constraints.align;
        materialize_arabic_tatweels_for_justified_line(
            &mut lines[index],
            &line_style,
            constraints.max_width,
            is_last_line,
            provider,
        );
    }
    for line in &mut lines {
        visual_order::apply_visual_order(line, visible_text, direction);
    }

    let mut resolved_lines = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let line_index = line_index_offset.saturating_add(index);
        let y = frame.y + line_index as f32 * line_height;
        let is_last_line = line_index.saturating_add(1) == total_line_count;
        let constraints = line_constraints[index];
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
    let measured_height = if virtualized {
        total_line_count as f32 * line_height
    } else {
        resolved_lines.len() as f32 * line_height
    };
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
        rich_text_artifact: None,
    }
}

fn resolve_overflow_style_with_provider(
    text: &str,
    style: &UiResolvedStyle,
    frame: UiFrame,
    provider: &mut SharedTextLayoutSession,
) -> UiResolvedStyle {
    let max_extent = if matches!(style.text_writing_mode, UiTextWritingMode::VerticalRl) {
        frame.height
    } else {
        frame.width
    };
    overflow_style::resolve(text, style, max_extent, |text, style| {
        UiSize::from(measure_backend_text_size_with_provider(
            text,
            &text_style(style),
            provider,
        ))
    })
}

#[cfg(test)]
mod tests;
