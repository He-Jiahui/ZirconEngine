use crate::text::font::{
    font_chain_line_metric_envelope, primary_face_covers_all_hard_line_content,
};
use crate::text::layout::{
    line_metrics_with_provider,
    measure_text_size_with_provider as measure_backend_text_size_with_provider,
    measure_text_source_range_width_with_provider as measure_backend_text_source_range_width_with_provider,
};
use crate::text::shaping::{TextLayoutOutcome, TextShapingOutcome};
use crate::text::{
    SharedTextLayoutSession, TextLayoutAxisConstraint, TextLayoutGeometryOwner, text_style,
};
use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiRichTextFormat, UiTextAlign, UiTextOverflow, UiTextRange, UiTextWrap,
    UiTextWritingMode,
};

use super::super::rich_text::parse_source_text_with_provider;
use super::geometry_admission::{
    validate_resolved_layout_geometry, validate_resolved_size_geometry,
};
use super::layout_parsed_text_with_provider_and_viewport_outcome;
use super::line_box::MIN_TEXT_FONT_SIZE;

pub(crate) fn measure_text_size(text: &str, style: &UiResolvedStyle) -> UiSize {
    let mut session = SharedTextLayoutSession::new();
    measure_text_size_with_provider(text, style, &mut session)
}

pub(crate) fn measure_text_size_with_provider(
    text: &str,
    style: &UiResolvedStyle,
    provider: &mut SharedTextLayoutSession,
) -> UiSize {
    match measure_text_size_with_provider_outcome(text, style, provider) {
        TextShapingOutcome::Ready(size) => size,
        TextShapingOutcome::Deferred(error) | TextShapingOutcome::Failed(error) => {
            provider.record_layout_error(&error);
            UiSize::default()
        }
    }
}

pub(crate) fn measure_text_size_with_provider_outcome(
    text: &str,
    style: &UiResolvedStyle,
    provider: &mut SharedTextLayoutSession,
) -> TextLayoutOutcome<UiSize> {
    let parsed =
        match parse_source_text_with_provider(text, style.rich_text_format.into(), provider) {
            Ok(parsed) => parsed,
            Err(error) => return TextShapingOutcome::failed(error),
        };
    if !matches!(
        style.rich_text_format,
        zircon_runtime_interface::ui::surface::UiRichTextFormat::Plain
    ) || matches!(style.text_writing_mode, UiTextWritingMode::VerticalRl)
    {
        let mut intrinsic_style = style.clone();
        intrinsic_style.wrap = UiTextWrap::None;
        intrinsic_style.text_overflow = UiTextOverflow::Clip;
        intrinsic_style.text_align = UiTextAlign::Left;
        let measurement_frame = match intrinsic_measurement_frame_with_provider(
            parsed.text(),
            &intrinsic_style,
            provider,
        ) {
            Ok(frame) => frame,
            Err(error) => return TextShapingOutcome::failed(error),
        };
        let layout = match layout_parsed_text_with_provider_and_viewport_outcome(
            &parsed,
            &intrinsic_style,
            measurement_frame,
            None,
            None,
            None,
            provider,
        ) {
            TextShapingOutcome::Ready(layout) => layout,
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        };
        if let Err(violation) =
            validate_resolved_layout_geometry(&layout, provider.geometry_budget())
        {
            return TextShapingOutcome::failed(provider.reject_geometry(
                TextLayoutGeometryOwner::IntrinsicMeasurement,
                violation,
                whole_source_range(parsed.text()),
                layout.lines.len().saturating_add(layout.boxes.len()),
            ));
        }
        return TextShapingOutcome::Ready(UiSize::new(
            layout.measured_width,
            layout.measured_height,
        ));
    }
    match measure_backend_text_size_with_provider(parsed.text(), &text_style(style), provider) {
        TextShapingOutcome::Ready(size) => {
            let size = UiSize::from(size);
            if let Err(violation) =
                validate_resolved_size_geometry(size, provider.geometry_budget())
            {
                return TextShapingOutcome::failed(provider.reject_geometry(
                    TextLayoutGeometryOwner::IntrinsicMeasurement,
                    violation,
                    whole_source_range(parsed.text()),
                    parsed.text().len(),
                ));
            }
            TextShapingOutcome::Ready(size)
        }
        TextShapingOutcome::Deferred(error) => TextShapingOutcome::Deferred(error),
        TextShapingOutcome::Failed(error) => TextShapingOutcome::Failed(error),
    }
}

/// Builds a non-painting unbounded frame for the rich and vertical intrinsic-measure paths.
///
/// Text03 defines positive infinity as an unbounded main-axis constraint. Vertical column
/// placement needs a finite cross axis, which is derived from canonical hard lines rather than
/// the source byte length.
pub(super) fn intrinsic_measurement_frame_with_provider(
    text: &str,
    style: &UiResolvedStyle,
    provider: &mut SharedTextLayoutSession,
) -> Result<UiFrame, crate::core::framework::text::TextLayoutError> {
    let unbounded = TextLayoutAxisConstraint::Unbounded;
    if matches!(style.text_writing_mode, UiTextWritingMode::VerticalRl) {
        let column_advance = style
            .line_height
            .max(style.font_size)
            .max(MIN_TEXT_FONT_SIZE);
        let column_count = crate::text::hard_line_count(text).max(1);
        let width = provider
            .geometry_budget()
            .checked_scale_accumulated(column_advance, column_count)
            .map_err(|violation| {
                provider.reject_geometry(
                    TextLayoutGeometryOwner::IntrinsicMeasurement,
                    violation,
                    whole_source_range(text),
                    column_count,
                )
            })?;
        return Ok(UiFrame::new(
            0.0,
            0.0,
            TextLayoutAxisConstraint::Bounded(width).request_extent(),
            unbounded.request_extent(),
        ));
    }

    Ok(UiFrame::new(
        0.0,
        0.0,
        unbounded.request_extent(),
        unbounded.request_extent(),
    ))
}

pub(super) fn bounded_inline_measurement_frame_with_provider(
    style: &UiResolvedStyle,
    origin_x: f32,
    origin_y: f32,
    inline_extent: f32,
    owner: TextLayoutGeometryOwner,
    source_range: Option<(u32, u32)>,
    work_units: usize,
    provider: &mut SharedTextLayoutSession,
) -> Result<UiFrame, crate::core::framework::text::TextLayoutError> {
    let budget = provider.geometry_budget();
    let constraint = TextLayoutAxisConstraint::from_request_extent(inline_extent, budget)
        .and_then(|constraint| match constraint {
            TextLayoutAxisConstraint::Bounded(_) => Ok(constraint),
            TextLayoutAxisConstraint::Unbounded => {
                budget.admit_axis_extent(inline_extent).map(|_| constraint)
            }
        })
        .map_err(|violation| {
            provider.reject_geometry(owner, violation, source_range, work_units)
        })?;
    budget.admit_coordinate(origin_x).map_err(|violation| {
        provider.reject_geometry(owner, violation, source_range, work_units)
    })?;
    budget.admit_coordinate(origin_y).map_err(|violation| {
        provider.reject_geometry(owner, violation, source_range, work_units)
    })?;
    let unbounded = TextLayoutAxisConstraint::Unbounded.request_extent();
    Ok(match style.text_writing_mode {
        UiTextWritingMode::VerticalRl => {
            UiFrame::new(origin_x, origin_y, unbounded, constraint.request_extent())
        }
        _ => UiFrame::new(origin_x, origin_y, constraint.request_extent(), unbounded),
    })
}

fn whole_source_range(text: &str) -> Option<(u32, u32)> {
    u32::try_from(text.len()).ok().map(|end| (0, end))
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

    let layout_style = text_style(style);
    let font_collection = provider.font_collection_snapshot();
    let font_revision = font_collection.revision();
    let database = font_collection.database();
    let line_height = if primary_face_covers_all_hard_line_content(database, &layout_style, text) {
        match line_metrics_with_provider(&layout_style, provider) {
            TextShapingOutcome::Ready(metrics) => metrics.line_height,
            TextShapingOutcome::Deferred(error) | TextShapingOutcome::Failed(error) => {
                provider.record_layout_error(&error);
                return None;
            }
        }
    } else {
        let requested_line_height = layout_style
            .line_height
            .max(layout_style.font_size.max(1.0));
        let Some(envelope) = font_chain_line_metric_envelope(database, &layout_style) else {
            return None;
        };
        if !envelope.certifies_uniform_line_height(requested_line_height) {
            return None;
        }
        requested_line_height
    };
    let line_count = provider
        .unretained_hard_line_count_and_window(text, 0..0)
        .0
        .max(1);
    let measured_height = match provider
        .geometry_budget()
        .checked_scale_accumulated(line_height, line_count)
    {
        Ok(height) => height,
        Err(violation) => {
            let error = provider.reject_geometry(
                TextLayoutGeometryOwner::IntrinsicMeasurement,
                violation,
                whole_source_range(text),
                line_count,
            );
            provider.record_layout_error(&error);
            return None;
        }
    };
    (provider.font_collection_revision() == font_revision).then_some(measured_height)
}

/// Certifies the fixed-height viewport path only when every source line uses the primary face.
/// A fallback-chain envelope can prove total height, but not the selected-face baseline required
/// to publish each visible physical line.
pub(super) fn certified_plain_viewport_line_height(
    text: &str,
    style: &UiResolvedStyle,
    sample_line_height: f32,
    provider: &SharedTextLayoutSession,
) -> Option<f32> {
    crate::profile_scope!(
        "runtime",
        "text.layout",
        "certify_plain_viewport_line_height"
    );
    if !matches!(style.rich_text_format, UiRichTextFormat::Plain)
        || !matches!(style.wrap, UiTextWrap::None)
        || !matches!(style.text_overflow, UiTextOverflow::Clip)
        || matches!(style.text_writing_mode, UiTextWritingMode::VerticalRl)
    {
        return None;
    }
    let layout_style = text_style(style);
    let font_collection = provider.font_collection_snapshot();
    let font_revision = font_collection.revision();
    primary_face_covers_all_hard_line_content(font_collection.database(), &layout_style, text)
        .then_some(sample_line_height)
        .filter(|line_height| line_height.is_finite() && *line_height > 0.0)
        .filter(|_| provider.font_collection_revision() == font_revision)
}

pub(crate) fn measure_text_source_range_width(
    text: &str,
    style: &UiResolvedStyle,
    range: UiTextRange,
) -> f32 {
    let mut session = SharedTextLayoutSession::new();
    let diagnostic_range = ui_source_range(range);
    let parsed =
        match parse_source_text_with_provider(text, style.rich_text_format.into(), &session) {
            Ok(parsed) => parsed,
            Err(error) => {
                session.record_layout_error(&error);
                return 0.0;
            }
        };
    match measure_backend_text_source_range_width_with_provider(
        parsed.text(),
        &text_style(style),
        range.into(),
        &mut session,
    ) {
        TextShapingOutcome::Ready(width) => {
            match session.geometry_budget().admit_axis_extent(width) {
                Ok(width) => width,
                Err(violation) => {
                    let error = session.reject_geometry(
                        TextLayoutGeometryOwner::IntrinsicMeasurement,
                        violation,
                        diagnostic_range,
                        range.end.saturating_sub(range.start),
                    );
                    session.record_layout_error(&error);
                    0.0
                }
            }
        }
        TextShapingOutcome::Deferred(error) | TextShapingOutcome::Failed(error) => {
            session.record_layout_error(&error);
            0.0
        }
    }
}

fn ui_source_range(range: UiTextRange) -> Option<(u32, u32)> {
    Some((
        u32::try_from(range.start).ok()?,
        u32::try_from(range.end).ok()?,
    ))
}
