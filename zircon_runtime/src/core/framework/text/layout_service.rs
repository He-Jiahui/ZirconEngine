use super::{
    TextDirection, TextFontRequest, TextLayoutError, TextRenderMode, TextShapeRequest,
    TextShapeResult,
};

pub trait TextLayoutService: Send + Sync {
    fn resolve_render_mode(&self, request: &TextFontRequest<'_>) -> TextRenderMode;

    fn resolve_direction(&self, text: &str, requested: TextDirection) -> TextDirection;

    fn shape(&self, request: TextShapeRequest<'_>) -> Result<TextShapeResult, TextLayoutError>;
}
