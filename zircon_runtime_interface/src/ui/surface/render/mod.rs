//! UI surface 渲染用的数据契约与后端无关的诊断视图导出入口；GPU 提交由 Runtime RHI 实现。
#[path = "batch.rs"]
mod batch;
#[path = "brush.rs"]
mod brush;
#[path = "cache.rs"]
mod cache;
#[path = "command.rs"]
mod command;
#[path = "command_kind.rs"]
mod command_kind;
#[path = "debug.rs"]
mod debug;
#[path = "editable_text.rs"]
mod editable_text;
#[path = "extract.rs"]
mod extract;
#[path = "frame_extract.rs"]
mod frame_extract;
#[path = "limits.rs"]
mod limits;
#[path = "list.rs"]
mod list;
#[path = "paint.rs"]
mod paint;
#[path = "parity.rs"]
mod parity;
#[path = "resolved_style.rs"]
mod resolved_style;
#[path = "text_effects.rs"]
mod text_effects;
#[path = "text_geometry/mod.rs"]
mod text_geometry;
#[path = "text_layout.rs"]
mod text_layout;
#[path = "text_shape.rs"]
mod text_shape;
#[path = "typography.rs"]
mod typography;
#[path = "visual_asset_ref.rs"]
mod visual_asset_ref;
#[path = "visualizer.rs"]
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
pub(crate) use parity::batch_indices_by_source_index;
pub use parity::{
    UiRendererParityBatchRow, UiRendererParityPaintRow, UiRendererParityPayloadKind,
    UiRendererParitySnapshot, UiRendererParityStats,
};
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
