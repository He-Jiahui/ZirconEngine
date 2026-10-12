//! Neutral text layout contracts shared by runtime consumers.

//! Runtime 提供布局与栅格服务实现；此模块公开调用方共享的排版服务接口、请求结果和字形栅格 DTO。
#[path = "direction.rs"]
mod direction;
#[path = "font_collection_handle.rs"]
mod font_collection_handle;
#[path = "font_face_handle.rs"]
mod font_face_handle;
#[path = "font_request.rs"]
mod font_request;
#[path = "glyph.rs"]
mod glyph;
#[path = "glyph_flags.rs"]
mod glyph_flags;
#[path = "glyph_raster/mod.rs"]
mod glyph_raster;
#[path = "glyph_rotation.rs"]
mod glyph_rotation;
#[path = "layout_error.rs"]
mod layout_error;
#[path = "layout_metrics.rs"]
mod layout_metrics;
#[path = "layout_service.rs"]
mod layout_service;
#[path = "open_type_feature.rs"]
mod open_type_feature;
#[path = "render_mode.rs"]
mod render_mode;
#[path = "shape_request.rs"]
mod shape_request;
#[path = "shape_result.rs"]
mod shape_result;
#[path = "shape_run.rs"]
mod shape_run;
#[path = "vertical_glyph_decision.rs"]
mod vertical_glyph_decision;
#[path = "writing_mode.rs"]
mod writing_mode;

pub use direction::TextDirection;
pub use font_collection_handle::TextFontCollectionHandle;
pub use font_face_handle::TextFontFaceHandle;
pub use font_request::TextFontRequest;
pub use glyph::TextGlyph;
pub use glyph_flags::TextGlyphFlags;
pub use glyph_raster::{
    TextGlyphBitmapFormat, TextGlyphRasterError, TextGlyphRasterHinting, TextGlyphRasterMode,
    TextGlyphRasterReceipt, TextGlyphRasterRequest, TextGlyphRasterSmoothing,
    TextGlyphSyntheticStyle,
};
pub use glyph_rotation::TextGlyphRotation;
pub use layout_error::TextLayoutError;
pub use layout_metrics::TextLayoutMetrics;
pub use layout_service::TextLayoutService;
pub use open_type_feature::TextOpenTypeFeature;
pub use render_mode::TextRenderMode;
pub use shape_request::TextShapeRequest;
pub use shape_result::TextShapeResult;
pub use shape_run::TextShapeRun;
pub use vertical_glyph_decision::{
    TextVerticalGlyphDecision, TextVerticalGlyphDecisionBasis, TextVerticalGlyphFallbackReason,
    TextVerticalGlyphFeatureSet, TextVerticalGlyphOrientation, TextVerticalGlyphSubstitution,
};
pub use writing_mode::TextWritingMode;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
