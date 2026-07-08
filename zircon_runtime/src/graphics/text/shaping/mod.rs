//! Shared text shaping owner. Third-party text backend types stay in leaf modules.

mod cosmic;
mod font_id;
mod line_break;
mod script_segment;

#[cfg(test)]
mod tests;

use std::sync::Arc;

use crate::core::framework::render::{ShapedGlyphRun, TextShapeRequest, TextShapingService};
use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextDirection, UiTextRange};

pub(crate) use font_id::{annotate_fallback_font_ids, font_query_for_style};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct SharedTextShapingService;

impl TextShapingService for SharedTextShapingService {
    fn shape_text(&self, request: TextShapeRequest<'_>) -> ShapedGlyphRun {
        cosmic::shape_text(request)
    }
}

pub(crate) fn shape_text(request: TextShapeRequest<'_>) -> ShapedGlyphRun {
    SharedTextShapingService.shape_text(request)
}

pub(crate) trait TextShapeRunProvider {
    fn shape_horizontal_line_with_kerning(
        &mut self,
        text: &str,
        style: &UiResolvedStyle,
        direction: UiTextDirection,
        source_range: UiTextRange,
        include_kerning: bool,
    ) -> Arc<ShapedGlyphRun>;
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct DirectTextShapeRunProvider;

impl TextShapeRunProvider for DirectTextShapeRunProvider {
    fn shape_horizontal_line_with_kerning(
        &mut self,
        text: &str,
        style: &UiResolvedStyle,
        direction: UiTextDirection,
        source_range: UiTextRange,
        include_kerning: bool,
    ) -> Arc<ShapedGlyphRun> {
        Arc::new(shape_horizontal_line_with_kerning(
            text,
            style,
            direction,
            source_range,
            include_kerning,
        ))
    }
}

pub(crate) fn shape_horizontal_line(
    text: &str,
    style: &UiResolvedStyle,
    direction: UiTextDirection,
    source_range: UiTextRange,
) -> ShapedGlyphRun {
    shape_horizontal_line_with_kerning(text, style, direction, source_range, true)
}

pub(crate) fn shape_horizontal_line_with_kerning(
    text: &str,
    style: &UiResolvedStyle,
    direction: UiTextDirection,
    source_range: UiTextRange,
    include_kerning: bool,
) -> ShapedGlyphRun {
    shape_text(TextShapeRequest::horizontal_with_kerning(
        text,
        style,
        direction,
        source_range,
        include_kerning,
    ))
}
