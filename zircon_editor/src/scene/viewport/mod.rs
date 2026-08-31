//! Scene viewport state, handle overlays, and editor-owned camera interaction.

mod controller;
mod edit_mode_projection;
mod handle_screen_line;
mod handles;
mod interaction;
mod interaction_extract;
pub(crate) mod pointer;
mod projection;
mod render_packet;
mod settings;

pub use crate::core::editing::interactive_transform::PivotMode;
pub(crate) use controller::{
    SceneViewportController, SceneViewportControllerError, ViewportOverlayProviderError,
};
pub(crate) use edit_mode_projection::{SceneEditModeProjection, SceneInspectorFieldValue};
pub(crate) use handle_screen_line::HandleScreenLine;
pub use handles::TransformHandleKind;
pub use interaction::{
    GizmoAxis, ViewportFeedback, ViewportInput, ViewportState, ViewportTransformRequest,
};
pub(in crate::scene::viewport) use interaction_extract::{
    ViewportInteractionExtract, ViewportInteractionExtractCache,
    ViewportInteractionExtractPointerResolution,
};
pub(in crate::scene::viewport) use settings::SceneViewportSnapSteps;
pub use settings::{
    GridMode, SceneViewportChromeSettings, SceneViewportSettings, TransformSpace, ViewOrientation,
};
// Runtime frame and renderer contracts consumed by the Editor viewport host.
pub(crate) use zircon_runtime::core::framework::render::{
    CapturedFrame, RenderFrameExtract, RenderFramework, RenderFrameworkError, RenderPipelineHandle,
    RenderQualityProfile, RenderStats, RenderWorldSnapshotHandle,
};
// Scene extract and overlay contracts projected through the viewport domain.
pub(crate) use zircon_runtime::core::framework::render::{
    FallbackSkyboxKind, GridOverlayExtract, HandleElementExtract, HandleOverlayExtract,
    OverlayAxis, OverlayBillboardIcon, OverlayLineSegment, OverlayPickShape, OverlayWireShape,
    PreviewEnvironmentExtract, RenderHybridGiExtract, RenderHybridGiProfile, RenderMeshSnapshot,
    RenderOverlayExtract, RenderSceneSnapshot, RenderVisibleSpatialQuerySnapshot, SceneGizmoKind,
    SceneGizmoOverlayExtract, SceneViewportExtractRequest, SelectionAnchorExtract,
};
// Viewport configuration, identity, and camera contracts shared by Editor surfaces.
pub(crate) use zircon_runtime::core::framework::render::{
    DisplayMode, ProjectionMode, RenderViewportDescriptor, RenderViewportHandle,
    RenderViewportProduct, ViewportCameraSnapshot, ViewportIconId,
};
