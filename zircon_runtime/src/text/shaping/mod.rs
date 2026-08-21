//! Shared text shaping owner. Third-party text backend types stay in leaf modules.

mod bidi;
mod cosmic;
mod fallback_spans;
mod horizontal;
mod itemize;
mod line_break;
mod normalize;
mod script_segment;
mod vertical;
mod work_budget;

#[cfg(test)]
mod tests;

use std::sync::Arc;

use crate::core::framework::text::TextDirection;
use crate::text::VerticalMode;
use crate::text::{BackendShapeRequest, ShapedGlyphRun};
use crate::text::{TextRange, TextStyle};

pub(crate) use bidi::{analyze_bidi_line, mirrored_bidi_char, resolve_bidi_base_direction};
pub(crate) use fallback_spans::{FallbackTextSpan, fallback_text_spans};
pub(crate) use vertical::{vertical_glyph_advance, vertical_glyph_rotation};
pub(crate) use work_budget::TextShapingWorkBudget;

pub(crate) fn shape_text(request: BackendShapeRequest<'_>) -> ShapedGlyphRun {
    let canonical_request = request.canonicalized();
    let request = canonical_request.request();
    cosmic::shape_text(request)
}

pub(crate) trait TextShapeRunProvider {
    fn shape_horizontal_line_with_kerning(
        &mut self,
        text: &str,
        style: &TextStyle,
        direction: TextDirection,
        source_range: TextRange,
        include_kerning: bool,
    ) -> Arc<ShapedGlyphRun>;

    fn shape_vertical_line_with_kerning(
        &mut self,
        text: &str,
        style: &TextStyle,
        direction: TextDirection,
        source_range: TextRange,
        vertical_mode: VerticalMode,
        include_kerning: bool,
    ) -> Arc<ShapedGlyphRun> {
        let _ = vertical_mode;
        self.shape_horizontal_line_with_kerning(
            text,
            style,
            direction,
            source_range,
            include_kerning,
        )
    }
}

pub(crate) struct VerticalTextShapeRunProvider<'a, P>
where
    P: TextShapeRunProvider + ?Sized,
{
    provider: &'a mut P,
    vertical_mode: VerticalMode,
}

impl<'a, P> VerticalTextShapeRunProvider<'a, P>
where
    P: TextShapeRunProvider + ?Sized,
{
    pub(crate) fn new(provider: &'a mut P, vertical_mode: VerticalMode) -> Self {
        Self {
            provider,
            vertical_mode,
        }
    }
}

impl<P> TextShapeRunProvider for VerticalTextShapeRunProvider<'_, P>
where
    P: TextShapeRunProvider + ?Sized,
{
    fn shape_horizontal_line_with_kerning(
        &mut self,
        text: &str,
        style: &TextStyle,
        direction: TextDirection,
        source_range: TextRange,
        include_kerning: bool,
    ) -> Arc<ShapedGlyphRun> {
        self.provider.shape_vertical_line_with_kerning(
            text,
            style,
            direction,
            source_range,
            self.vertical_mode,
            include_kerning,
        )
    }

    fn shape_vertical_line_with_kerning(
        &mut self,
        text: &str,
        style: &TextStyle,
        direction: TextDirection,
        source_range: TextRange,
        vertical_mode: VerticalMode,
        include_kerning: bool,
    ) -> Arc<ShapedGlyphRun> {
        self.provider.shape_vertical_line_with_kerning(
            text,
            style,
            direction,
            source_range,
            vertical_mode,
            include_kerning,
        )
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct DirectTextShapeRunProvider;

impl TextShapeRunProvider for DirectTextShapeRunProvider {
    fn shape_horizontal_line_with_kerning(
        &mut self,
        text: &str,
        style: &TextStyle,
        direction: TextDirection,
        source_range: TextRange,
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

    fn shape_vertical_line_with_kerning(
        &mut self,
        text: &str,
        style: &TextStyle,
        direction: TextDirection,
        source_range: TextRange,
        vertical_mode: VerticalMode,
        include_kerning: bool,
    ) -> Arc<ShapedGlyphRun> {
        Arc::new(shape_text(BackendShapeRequest::vertical_with_kerning(
            text,
            style,
            direction,
            source_range,
            vertical_mode,
            include_kerning,
        )))
    }
}

pub(crate) fn shape_horizontal_line(
    text: &str,
    style: &TextStyle,
    direction: TextDirection,
    source_range: TextRange,
) -> ShapedGlyphRun {
    shape_horizontal_line_with_kerning(text, style, direction, source_range, true)
}

pub(crate) fn shape_horizontal_line_with_kerning(
    text: &str,
    style: &TextStyle,
    direction: TextDirection,
    source_range: TextRange,
    include_kerning: bool,
) -> ShapedGlyphRun {
    shape_text(BackendShapeRequest::horizontal_with_kerning(
        text,
        style,
        direction,
        source_range,
        include_kerning,
    ))
}
