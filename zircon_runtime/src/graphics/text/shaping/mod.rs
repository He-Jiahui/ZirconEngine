//! Shared text shaping owner. Third-party text backend types stay in leaf modules.

mod cosmic;
mod line_break;
mod script_segment;

#[cfg(test)]
mod tests;

use crate::core::framework::render::{ShapedGlyphRun, TextShapeRequest, TextShapingService};
use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextDirection, UiTextRange};

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

pub(crate) fn shape_horizontal_line(
    text: &str,
    style: &UiResolvedStyle,
    direction: UiTextDirection,
    source_range: UiTextRange,
) -> ShapedGlyphRun {
    shape_text(TextShapeRequest::horizontal(
        text,
        style,
        direction,
        source_range,
    ))
}
