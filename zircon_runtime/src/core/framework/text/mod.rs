//! Neutral text layout contracts shared by runtime consumers.

//! Runtime 提供布局与栅格服务实现；此模块公开调用方共享的排版服务接口、请求结果和字形栅格 DTO。
mod direction;
mod font_collection_handle;
mod font_face_handle;
mod font_request;
mod glyph;
mod glyph_flags;
mod glyph_raster;
mod glyph_rotation;
mod layout_error;
mod layout_metrics;
mod layout_service;
mod open_type_feature;
mod render_mode;
mod shape_request;
mod shape_result;
mod shape_run;
mod vertical_glyph_decision;
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
