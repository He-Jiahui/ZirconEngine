//! Shared runtime text services used by UI and render-side text consumers.

mod glyph_artifact;
mod hard_line;
mod language;
mod layout_session;
mod model;
mod native_buffer;
mod render_state;
mod service;
mod ui_style;

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
    InlineBaseline, InlineObjectRef, Iso15924Tag, LaidOutLine, LaidOutText, LayoutItem, LinkRef,
    MAX_RICH_TABLE_ROW_SPAN, OpenTypeFeature, ParagraphOverride, RichParseResult, RichTable,
    RichTableCell, RichTableCellBoxStyle, RichTableCellPadding, RichTableColumn, RichTextFormat,
    ShapedGlyph, ShapedGlyphClusterFlags, ShapedGlyphRotation, ShapedGlyphRun, ShapedGlyphScript,
    ShapedTextLine, StyleOverride, StyledRun, TextAlign, TextOrientation, TextRange, TextWrap,
    VerticalMode, normalized_open_type_features,
};

pub(crate) use cache::TextDocumentKey;
pub use cache::{
    CompiledRichTextCacheFrameSampler, CompiledRichTextCacheReport,
    shared_compiled_rich_text_cache_report,
};
pub(crate) use glyph_artifact::{
    ResolvedTextGlyphArtifact, ResolvedTextGlyphArtifactLine, build_resolved_text_glyph_artifact,
    build_resolved_text_glyph_artifact_with_shared_source,
    rebuild_resolved_text_glyph_artifact_line, register_resolved_text_glyph_artifact,
    resolve_resolved_text_glyph_artifact, resolved_text_glyph_artifact_caret_advance,
    resolved_text_glyph_artifact_caret_at_advance,
    resolved_text_glyph_artifact_range_advance_spans, resolved_text_line_requires_visual_fallback,
};
pub(crate) use hard_line::{
    HardLine, TEXT_SHAPING_RUN_MAX_BYTES, hard_line_count, hard_line_count_and_window,
    hard_line_end, hard_line_start, hard_line_window, hard_lines, next_hard_line_start,
    visit_hard_lines,
};
pub(crate) use language::{default_text_locale, normalize_text_language_tag, system_text_locale};
pub(crate) use layout_session::SharedTextLayoutSession;
pub use layout_session::{TextLayoutFallbackReport, shared_text_layout_fallback_report};
pub(crate) use model::{BackendShapeRequest, TextFrame, TextSize, TextStyle};
pub(crate) use native_buffer::{NativeTextAlign, NativeTextBufferRequest, NativeTextWrap};
pub(crate) use render_state::TextRenderState;
pub use rich::{
    CompiledRichText, EmojiShortcodeRegistrationError, RichTextDecoration, RichTextDecorator,
    RichTextDecoratorRegistrationError, RichTextParser,
};
pub(crate) use rich::{register_compiled_rich_text_artifact, resolve_compiled_rich_text_artifact};
pub use service::{SharedTextLayoutService, shared_text_layout_service};
pub(crate) use service::{
    TextLayoutGenerationRetryReport, fallback_spans_for_request,
    shared_text_layout_generation_retry_report,
};
pub(crate) use ui_style::text_style;
