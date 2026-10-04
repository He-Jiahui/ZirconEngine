mod backend;
mod composition;
mod direct;

use crate::text::layout_geometry::FiniteGeometryAccumulator;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

pub(super) use backend::{shape_horizontal_run, HorizontalBackendRun};
pub(super) use composition::{
    compose_horizontal_partial, HorizontalDirectShapeAttempt, HorizontalPartialShape,
};
pub(super) use direct::shape_horizontal_request;

pub(super) fn position_glyphs(glyphs: &mut [crate::text::ShapedGlyph]) -> f32 {
    let mut cursor = FiniteGeometryAccumulator::default();
    for glyph in glyphs {
        glyph.x = cursor.value();
        let advance = glyph.advance.max(0.0);
        cursor.add(advance);
    }
    cursor.value()
}
