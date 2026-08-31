//! Shared runtime text services used by UI and render-side text consumers.

pub(crate) mod document;
mod glyph_artifact;
mod hard_line;
mod identity;
mod joining_type;
mod language;
mod layout_geometry;
mod layout_session;
mod model;
mod module;
mod render_state;
mod runtime_artifact;
mod semantic_projection;
mod service;
mod ui_style;
mod unicode_data;
mod word_boundary;

pub(crate) mod atlas;
pub(crate) mod cache;
mod cluster_geometry;
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

pub use crate::core::framework::text::{
    TextVerticalGlyphDecision, TextVerticalGlyphDecisionBasis, TextVerticalGlyphFallbackReason,
    TextVerticalGlyphFeatureSet, TextVerticalGlyphOrientation, TextVerticalGlyphSubstitution,
};
pub use model::font::{
    CompositeFontDescriptor, FaceIndex, FontCultureTag, FontFaceDescriptor, FontFaceId,
    FontFamilyDescriptor, FontFamilyName, FontMatch, FontQuery, FontScript, FontScriptTag,
    FontStretch, FontStyle, FontWeight, InstancedFaceId, SubFontRange, VariationCoords,
};
pub use model::{
    InlineBaseline, InlineObjectRef, Iso15924Tag, LaidOutLine, LaidOutText, LayoutItem,
    LineBreakTailoringProfile, LinkRef, MAX_RICH_TABLE_ROW_SPAN, OpenTypeFeature,
    ParagraphOverride, RichIconAssetId, RichInlineWidgetSlotId, RichListItem, RichListItemKind,
    RichOrderedListMarker, RichParseResult, RichTable, RichTableCell, RichTableCellBoxStyle,
    RichTableCellPadding, RichTableColumn, RichTextAuthoringDiagnostic,
    RichTextAuthoringDiagnosticCode, RichTextAuthoringDiagnosticSeverity,
    RichTextAuthoringRecovery, RichTextFormat, ShapedGlyph, ShapedGlyphBreakSafety,
    ShapedGlyphClusterFlags, ShapedGlyphLineBreakOpportunity, ShapedGlyphLineBreakReceipt,
    ShapedGlyphRotation, ShapedGlyphRun, ShapedGlyphScript, ShapedHardLine, StyleOverride,
    StyledRun, TextAlign, TextHorizontalCompositionReceipt, TextOrientation, TextRange, TextWrap,
    VerticalGlyphDecision, VerticalMode, normalized_open_type_features,
};
pub(crate) use module::font_collection_service_for_core;
pub use module::{TEXT_MODULE_NAME, TextModule};

pub use cache::CompiledRichTextCacheReport;
pub(crate) use cache::TextDocumentKey;
pub(crate) use cluster_geometry::text_glyph_clusters;
pub(crate) use glyph_artifact::{
    BuiltResolvedRichTextGlyphArtifact, ResolvedTextGlyphArtifact,
    ResolvedTextGlyphArtifactFontLease, ResolvedTextGlyphArtifactLine,
    build_resolved_rich_text_glyph_artifact, build_resolved_text_glyph_artifact,
    build_resolved_text_glyph_artifact_with_line_fragments,
    build_resolved_text_glyph_artifact_with_shared_source,
    build_resolved_text_presentation_glyph_artifact, register_resolved_text_glyph_artifact,
    resolve_resolved_text_glyph_artifact, resolved_text_glyph_artifact_caret_advance,
    resolved_text_glyph_artifact_caret_at_advance,
    resolved_text_glyph_artifact_line_matches_layout,
    resolved_text_glyph_artifact_matches_layout_snapshot,
    resolved_text_glyph_artifact_range_advance_spans, resolved_text_line_requires_visual_fallback,
};
pub(crate) use hard_line::{
    HardLine, hard_line_count, hard_line_count_and_window, hard_line_end, hard_line_start,
    hard_line_window, hard_lines, has_multiple_hard_lines, is_hard_line_separator,
    next_hard_line_start, visit_hard_lines,
};
pub(crate) use identity::{EphemeralCacheHash, EphemeralCacheHasher, StableContentDigest};
pub(crate) use joining_type::{TextJoiningTypeMap, compiled_joining_type_map};
pub(crate) use language::{
    default_text_locale, normalize_text_language_tag, system_text_locale,
    text_language_cache_identity,
};
pub(crate) use layout_geometry::{
    TextLayoutAxisConstraint, TextLayoutGeometryBudget, TextLayoutGeometryOwner,
    TextLayoutGeometryViolation,
};
pub use layout_session::TextLayoutFallbackReport;
#[cfg(test)]
pub(crate) use layout_session::current_thread_text_layout_session_construction_count;
pub(crate) use layout_session::{
    SharedTextLayoutSession, TextLayoutGeometryRejectionReceipt, TextLayoutGeometryReport,
    TextLayoutSessionDiagnostics, TextTableLayoutWorkReport,
};
pub(crate) use model::{
    BackendShapeRequest, HorizontalGlyphMetricSpan, HorizontalLineRawMetrics, TextFrame, TextSize,
    TextStyle,
};
pub(crate) use render_state::TextRenderState;
pub use rich::{
    CompiledRichText, EmojiShortcodeRegistrationError, RichParseBudget, RichTextContentTrust,
    RichTextDecoration, RichTextDecorator, RichTextDecoratorRegistrationError, RichTextDependency,
    RichTextParseError, RichTextParser,
};
pub(crate) use rich::{register_compiled_rich_text_artifact, resolve_compiled_rich_text_artifact};
pub(crate) use runtime_artifact::{
    ResolvedRichTextGlyphRun, ResolvedRichTextGlyphRunArtifact,
    register_resolved_rich_text_artifact_with_layout_runs, resolve_rich_text_glyph_run_artifact,
    resolve_rich_text_glyph_run_artifact_at, resolve_rich_text_virtual_line_sequences_for_layout,
    resolved_rich_text_artifact_matches_layout_snapshot,
};
pub(crate) use semantic_projection::{
    RichSemanticProjection, from_compiled_rich_semantic_projection,
    resolve_rich_semantic_projection,
};
pub use service::{SharedTextLayoutService, shared_text_layout_service};
pub(crate) use service::{
    TextLayoutGenerationRetryReport, fallback_spans_for_request,
    shape_text_request_in_font_collection, shared_text_layout_generation_retry_report,
};
pub use shaping::{
    TextShapingBudgetKind, TextShapingFailureCode, TextShapingFailureDependency,
    TextShapingFailureDisposition, TextShapingFailurePhase, TextShapingFailureReceipt,
    TextShapingFailureReport,
};
pub(crate) use shaping::{TextShapingWorkBudget, TextShapingWorkReport};
pub(crate) use ui_style::text_style;
pub use unicode_data::{
    TextDataVersion, UnicodeDataSnapshot, UnicodeDataSnapshotId, UnicodeProviderSnapshot,
    compiled_unicode_data_snapshot, compiled_unicode_data_snapshot_id,
};
pub(crate) use word_boundary::WordBoundaryMap;
