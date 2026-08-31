use crate::text::layout::{TextLineMetrics, line_metrics_with_provider};
use crate::text::shaping::{TextLayoutOutcome, TextShapingOutcome};
use crate::text::{
    SharedTextLayoutSession, TextDocumentKey, build_resolved_rich_text_glyph_artifact,
    build_resolved_text_glyph_artifact_with_line_fragments, register_compiled_rich_text_artifact,
    register_resolved_rich_text_artifact_with_layout_runs, text_style,
};
use std::sync::Arc;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine, UiTextOverflow, UiTextRange,
    UiTextWritingMode,
};

use super::rich_text::{UiParsedText, parse_source_text_with_provider};

mod artifact;
mod candidate_line;
mod direction;
mod ellipsis;
mod failure_layout;
mod geometry_admission;
mod layout_result;
mod line_box;
mod measurement;
mod overflow_style;
mod paragraph_layout;
mod physical_line_metrics;
mod plain_layout;
mod range_mapping;
mod rich_layout;
mod rich_layout_vertical;
mod rich_table;
mod secure_presentation;
mod vertical;
mod viewport;
mod virtual_fragment_sequence;
mod visual_order;
mod wrapping;

use super::resolved_layout::UiTextViewport;
use artifact::{LayoutFontGenerationFence, attach_plain_text_glyph_artifact};
use ellipsis::{
    ellipsize_line_with_provider, is_ellipsis_overflow, line_overflows_horizontally_with_provider,
    merge_clipped_lines_for_tail_preserving_ellipsis,
};
pub(super) use failure_layout::text_layout_error_layout;
use layout_result::LayoutWithoutArtifact;
use line_box::{
    MIN_TEXT_FONT_SIZE, aligned_x, available_wrap_extent,
    materialize_arabic_tatweels_for_justified_line, resolve_line_widths_with_provider,
};
use viewport::visible_plain_text_lines;
use wrapping::wrap_source_runs_with_provider;

pub(crate) use direction::resolve_direction as resolve_text_direction;
pub(crate) use measurement::{
    measure_text_size, measure_text_size_with_provider, measure_text_size_with_provider_outcome,
    measure_text_source_range_width, measure_unwrapped_text_height,
    measure_unwrapped_text_height_with_provider,
};
pub(crate) use secure_presentation::apply_secure_text_presentation;

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
    let line_height = match line_metrics_with_provider(&text_style(style), provider) {
        TextShapingOutcome::Ready(metrics) => metrics.line_height,
        TextShapingOutcome::Deferred(error) | TextShapingOutcome::Failed(error) => {
            provider.record_layout_error(&error);
            return false;
        }
    };
    visible_plain_text_lines(parsed, style, viewport, line_height, document_key, provider).is_some()
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
    let parsed =
        match parse_source_text_with_provider(text, style.rich_text_format.into(), provider) {
            Ok(parsed) => parsed,
            Err(error) => {
                return text_layout_error_layout(
                    style,
                    resolve_text_direction(text, style.text_direction),
                    style.font_size.max(MIN_TEXT_FONT_SIZE),
                    style
                        .line_height
                        .max(style.font_size)
                        .max(MIN_TEXT_FONT_SIZE),
                    text.len(),
                    &error,
                    provider,
                );
            }
        };
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

pub(super) fn layout_parsed_text_with_provider_and_viewport_outcome(
    parsed: &super::rich_text::UiParsedText,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    viewport: Option<UiTextViewport>,
    document_key: Option<TextDocumentKey>,
    provider: &mut SharedTextLayoutSession,
) -> TextLayoutOutcome<UiResolvedTextLayout> {
    let font_generation_fence = LayoutFontGenerationFence::capture(provider);
    let collect_profile_metrics = layout_profile_metrics_enabled();
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    let cache_report_before = collect_profile_metrics.then(|| provider.cache_report());
    let layout_outcome = {
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
    if let Some(cache_report_before) = cache_report_before {
        let cache_report_after = provider.cache_report();
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
    let layout_without_artifact = match layout_outcome.and_then(|layout| {
        font_generation_fence
            .ensure_current(provider)
            .map(|()| layout)
    }) {
        TextShapingOutcome::Ready(layout) => layout,
        TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
        TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
    };
    let mut layout = layout_without_artifact.layout;
    if !matches!(
        style.rich_text_format,
        zircon_runtime_interface::ui::surface::UiRichTextFormat::Plain
    ) {
        layout.rich_text_artifact = match build_resolved_rich_text_glyph_artifact(
            parsed,
            parsed.rich.shared_text(),
            style,
            &layout,
            layout_without_artifact
                .retained_virtual_line_sequences
                .as_deref(),
            provider,
        ) {
            TextShapingOutcome::Ready(Some(artifact)) => {
                Some(register_resolved_rich_text_artifact_with_layout_runs(
                    Arc::clone(&parsed.rich),
                    artifact.artifact,
                    Arc::from(layout.lines.clone()),
                    artifact.glyph_runs,
                ))
            }
            TextShapingOutcome::Ready(None) => Some(register_compiled_rich_text_artifact(
                Arc::clone(&parsed.rich),
            )),
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        };
    } else {
        let retained_line_fragments = layout_without_artifact.retained_line_fragments.as_deref();
        let retained_virtual_line_sequences = layout_without_artifact
            .retained_virtual_line_sequences
            .as_deref();
        let artifact = if viewport.is_some() && parsed.source_offset() == 0 {
            build_resolved_text_glyph_artifact_with_line_fragments(
                parsed.rich.shared_text(),
                style,
                &layout,
                retained_line_fragments,
                retained_virtual_line_sequences,
                provider,
            )
        } else {
            build_resolved_text_glyph_artifact_with_line_fragments(
                Arc::from(parsed.text()),
                style,
                &layout,
                retained_line_fragments,
                retained_virtual_line_sequences,
                provider,
            )
        };
        match attach_plain_text_glyph_artifact(&mut layout, artifact) {
            TextShapingOutcome::Ready(()) => {}
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        }
    }
    font_generation_fence
        .ensure_current(provider)
        .map(|()| layout)
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
    match layout_parsed_text_with_provider_and_viewport_outcome(
        parsed,
        style,
        frame,
        clip_frame,
        viewport,
        document_key,
        provider,
    ) {
        TextShapingOutcome::Ready(layout) => layout,
        TextShapingOutcome::Deferred(error) | TextShapingOutcome::Failed(error) => {
            text_layout_error_layout(
                style,
                resolve_text_direction(parsed.text(), style.text_direction),
                style.font_size.max(MIN_TEXT_FONT_SIZE),
                style
                    .line_height
                    .max(style.font_size)
                    .max(MIN_TEXT_FONT_SIZE),
                parsed.text().len(),
                &error,
                provider,
            )
        }
    }
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

pub(super) fn layout_parsed_text_with_provider_outcome(
    parsed: &super::rich_text::UiParsedText,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    provider: &mut SharedTextLayoutSession,
) -> TextLayoutOutcome<UiResolvedTextLayout> {
    layout_parsed_text_with_provider_and_viewport_outcome(
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
) -> TextLayoutOutcome<LayoutWithoutArtifact> {
    match rich_table::layout_rich_tables_with_provider(parsed, style, frame, clip_frame, provider) {
        TextShapingOutcome::Ready(Some(layout)) => {
            return TextShapingOutcome::Ready(LayoutWithoutArtifact::without_retained_fragments(
                layout,
            ));
        }
        TextShapingOutcome::Ready(None) => {}
        TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
        TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
    }
    plain_layout::layout_parsed_text_without_tables_with_retained_fragments(
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
) -> TextLayoutOutcome<UiResolvedTextLayout> {
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
) -> TextLayoutOutcome<UiResolvedTextLayout> {
    plain_layout::layout_parsed_text_without_tables_with_retained_fragments(
        parsed,
        style,
        frame,
        clip_frame,
        viewport,
        document_key,
        provider,
    )
    .map(|result| result.layout)
}

#[cfg(test)]
mod tests;
