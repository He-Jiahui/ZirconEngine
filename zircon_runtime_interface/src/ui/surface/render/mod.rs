mod batch;
mod brush;
mod cache;
mod command;
mod command_kind;
mod debug;
mod editable_text;
mod extract;
mod frame_extract;
mod limits;
mod list;
mod paint;
mod parity;
mod resolved_style;
mod text_effects;
mod text_geometry;
mod text_layout;
mod text_shape;
mod typography;
mod visual_asset_ref;
mod visualizer;

pub use batch::{
    UiBatch, UiBatchKey, UiBatchPlan, UiBatchPrimitive, UiBatchRange, UiBatchShader,
    UiBatchSplitReason, UiBatchStats, UiOpacityClass,
};
pub use brush::{
    UiBorderBrushPayload, UiBrushPayload, UiBrushSet, UiGradientBrushPayload, UiGradientStop,
    UiImageBrushPayload, UiMaterialBrushPayload, UiRenderResourceKey, UiRenderResourceKind,
    UiRenderResourceState, UiResourceUvRect, UiRoundedBrushPayload, UiSolidBrushPayload,
    UiVectorBrushPayload,
};
pub use cache::{
    UiRenderCacheBatchEntry, UiRenderCacheInvalidationReason, UiRenderCachePaintEntry,
    UiRenderCachePlan, UiRenderCacheStats, UiRenderCacheStatus,
};
pub use command::UiRenderCommand;
pub use command_kind::UiRenderCommandKind;
pub use debug::{UiRenderBatchDebugEntry, UiRenderDebugSnapshot, UiRenderDebugStatsV2};
pub use editable_text::{
    UiEditableTextState, UiTextByteRange, UiTextCaret, UiTextCaretAffinity, UiTextComposition,
    UiTextEditAction, UiTextPreeditClause, UiTextPreeditClauseError, UiTextPreeditClauseKind,
    UiTextSelection,
};
pub use extract::{UiRenderExtract, UiRenderExtractKind, UiRenderStats};
pub use frame_extract::{
    UiRenderFrameCommandRef, UiRenderFrameCommands, UiRenderFrameExtract, UiRenderFrameList,
    UiRenderFramePatchStats, UI_RENDER_FRAME_COMMAND_SEGMENT_SIZE,
};
pub use limits::{
    bounded_ui_slider_tick_count, ui_slider_tick_count_for_track, MAX_UI_SLIDER_TICK_COUNT,
};
pub use list::UiRenderList;
pub use paint::{
    UiClipMode, UiClipState, UiDrawEffect, UiPaintEffects, UiPaintElement, UiPaintPayload,
};
pub use parity::{
    UiRendererParityBatchRow, UiRendererParityPaintRow, UiRendererParityPayloadKind,
    UiRendererParitySnapshot, UiRendererParityStats,
};
pub(crate) use parity::batch_indices_by_source_index;
pub use resolved_style::UiResolvedStyle;
pub use text_effects::{
    UiTextDecorations, UiTextDistanceFieldEffects, UiTextGlowEffect, UiTextOutlineEffect,
    UiTextShadowEffect, MAX_TEXT_EFFECT_EXTENT_PX,
};
pub use text_geometry::{UiTextLineSourceMap, UiTextVisualBoundaryBias, UiTextVisualSpan};
pub use text_layout::{
    UiResolvedTextBox, UiResolvedTextLayout, UiResolvedTextLine, UiResolvedTextRun,
    UiRichTextArtifactHandle, UiTextRange,
};
pub use text_shape::{
    UiShapedGlyph, UiShapedGlyphClusterFlags, UiShapedGlyphRotation, UiShapedText,
    UiShapedTextCluster, UiShapedTextLine, UiTextPaint, UiTextPaintDecoration,
    UiTextPaintDecorationKind, UiTextPaintRun, UiTextRunPaintStyle, UiTextShapeArtifact,
};
pub use typography::{
    resolve_ui_text_render_mode, UiRichTextFormat, UiTextAlign, UiTextDirection, UiTextOverflow,
    UiTextRenderMode, UiTextRunKind, UiTextWrap, UiTextWritingMode,
};
pub use visual_asset_ref::UiVisualAssetRef;
pub use visualizer::{
    UiRenderVisualizerBatchGroup, UiRenderVisualizerOverdrawRegion, UiRenderVisualizerOverlay,
    UiRenderVisualizerOverlayKind, UiRenderVisualizerPaintElement,
    UiRenderVisualizerPaintPayloadKind, UiRenderVisualizerResourceBinding,
    UiRenderVisualizerSnapshot, UiRenderVisualizerStats, UiRenderVisualizerTextStats,
};
