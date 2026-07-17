//! Shared runtime text services used by UI and render-side text consumers.

mod language;
mod layout_session;
mod model;
mod native_buffer;
mod render_state;
mod service;

pub(crate) mod atlas;
pub(crate) mod cache;
pub(crate) mod font;
#[cfg(feature = "font-sdf-build-tool")]
pub mod font_sdf_build_tool;
pub(crate) mod layout;
pub(crate) mod native_bitmap_atlas;
pub(crate) mod parallel;
pub(crate) mod raster;
pub(crate) mod rich;
pub(crate) mod sdf;
pub(crate) mod shaping;

pub use model::font::{
    CompositeFontDescriptor, FaceIndex, FontCultureTag, FontFaceDescriptor, FontFaceId,
    FontFamilyDescriptor, FontFamilyName, FontMatch, FontQuery, FontScript, FontStretch, FontStyle,
    FontWeight, InstancedFaceId, SubFontRange, VariationCoords,
};
pub use model::{
    normalized_open_type_features, InlineBaseline, InlineObjectRef, LaidOutLine, LaidOutText,
    LayoutItem, LinkRef, OpenTypeFeature, ParagraphOverride, RichParseResult, RichTable,
    RichTableCell, RichTableCellBoxStyle, RichTableCellPadding, RichTableColumn, RichTextFormat,
    ShapedGlyph, ShapedGlyphClusterFlags, ShapedGlyphRotation, ShapedGlyphRun, ShapedGlyphScript,
    ShapedTextLine, StyleOverride, StyledRun, TextAlign, TextOrientation, TextRange, TextWrap,
    VerticalMode, MAX_RICH_TABLE_ROW_SPAN,
};

pub(crate) use language::normalize_text_language_tag;
pub(crate) use layout_session::SharedTextLayoutSession;
pub use layout_session::{shared_text_layout_fallback_report, TextLayoutFallbackReport};
pub(crate) use model::{BackendShapeRequest, TextFrame, TextSize, TextStyle};
pub(crate) use native_buffer::{NativeTextAlign, NativeTextBufferRequest, NativeTextWrap};
pub(crate) use render_state::TextRenderState;
pub use rich::{
    EmojiShortcodeRegistrationError, RichTextDecoration, RichTextDecorator,
    RichTextDecoratorRegistrationError, RichTextParser,
};
pub(crate) use service::fallback_spans_for_request;
pub use service::{shared_text_layout_service, SharedTextLayoutService};
