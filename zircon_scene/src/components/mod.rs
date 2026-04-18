//! ECS-style components, scheduling, viewport state, and render extraction snapshots.

mod gizmo;
mod render_extract;
mod scene;
mod schedule;
mod viewport;

pub use gizmo::{
    GridOverlayExtract, HandleElementExtract, HandleOverlayExtract, OverlayAxis,
    OverlayBillboardIcon, OverlayLineSegment, OverlayPickShape, OverlayWireShape,
    RenderOverlayExtract, SceneGizmoBuildContext, SceneGizmoKind, SceneGizmoOverlayExtract,
    SceneGizmoProvider, SceneGizmoRegistry, SelectionAnchorExtract, SelectionHighlightExtract,
    ViewportIconId,
};
pub use render_extract::{
    PreviewEnvironmentExtract, RenderBakedLightingExtract, RenderBloomSettings,
    RenderColorGradingSettings, RenderDirectionalLightSnapshot, RenderExtractPacket,
    RenderHybridGiExtract, RenderHybridGiProbe, RenderHybridGiTraceRegion, RenderMeshSnapshot,
    RenderParticleSpriteSnapshot, RenderReflectionProbeSnapshot, RenderSceneGeometryExtract,
    RenderSceneSnapshot, RenderVirtualGeometryCluster, RenderVirtualGeometryExtract,
    RenderVirtualGeometryPage, SceneViewportRenderPacket,
};
pub use scene::{
    default_render_layer_mask, Active, ActiveInHierarchy, ActiveSelf, CameraComponent,
    DirectionalLight, Hierarchy, LocalTransform, MeshRenderer, Mobility, Name, NodeKind,
    NodeRecord, RenderLayerMask, SceneNode, WorldMatrix, WorldTransform,
};
pub use schedule::{Schedule, SystemStage};
pub use viewport::{
    aspect_ratio_from_viewport_size, default_viewport_aspect_ratio, DisplayMode,
    FallbackSkyboxKind, GridMode, ProjectionMode, SceneViewportExtractRequest,
    SceneViewportSettings, SceneViewportTool, TransformSpace, ViewOrientation,
    ViewportCameraSnapshot,
};
