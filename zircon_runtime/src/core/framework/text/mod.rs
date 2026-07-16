//! Neutral text layout contracts shared by runtime consumers.

mod direction;
mod font_face_handle;
mod font_request;
mod glyph;
mod glyph_flags;
mod glyph_rotation;
mod layout_error;
mod layout_metrics;
mod layout_service;
mod render_mode;
mod shape_request;
mod shape_result;
mod shape_run;
mod writing_mode;

pub use direction::TextDirection;
pub use font_face_handle::TextFontFaceHandle;
pub use font_request::TextFontRequest;
pub use glyph::TextGlyph;
pub use glyph_flags::TextGlyphFlags;
pub use glyph_rotation::TextGlyphRotation;
pub use layout_error::TextLayoutError;
pub use layout_metrics::TextLayoutMetrics;
pub use layout_service::TextLayoutService;
pub use render_mode::TextRenderMode;
pub use shape_request::TextShapeRequest;
pub use shape_result::TextShapeResult;
pub use shape_run::TextShapeRun;
pub use writing_mode::TextWritingMode;

#[cfg(test)]
mod tests;
