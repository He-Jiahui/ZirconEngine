//! Scene viewport state, handle overlays, and editor-owned camera interaction.

#[path = "controller/mod.rs"]
mod controller;
#[path = "edit_mode_projection/mod.rs"]
mod edit_mode_projection;
#[path = "handle_screen_line.rs"]
mod handle_screen_line;
#[path = "handles/mod.rs"]
mod handles;
#[path = "interaction/mod.rs"]
mod interaction;
#[path = "interaction_extract/mod.rs"]
mod interaction_extract;
#[path = "pointer/mod.rs"]
pub(crate) mod pointer;
#[path = "projection.rs"]
mod projection;
#[path = "render_packet.rs"]
mod render_packet;
#[path = "settings.rs"]
mod settings;

pub use crate::core::editing::interactive_transform::PivotMode;
pub(crate) use controller::{
    SceneViewportCameraSnapshot, SceneViewportController, SceneViewportControllerError,
    SceneViewportSessionRegistry, SceneViewportWorkspaceSessionSnapshot,
    ViewportOverlayProviderError,
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
