use super::shaped_run::{ShapedGlyphRun, TextShapeRequest};

pub trait TextShapingService {
    fn shape_text(&self, request: TextShapeRequest<'_>) -> ShapedGlyphRun;
}
