//! ECS-style components plus framework-owned viewport and render snapshot types.

mod scene;
mod schedule;

pub use scene::{
    default_render_layer_mask, Active, ActiveInHierarchy, ActiveSelf, CameraComponent, DirectionalLight,
    Hierarchy, LocalTransform, MeshRenderer, Name, NodeKind, NodeRecord, RenderLayerMask,
    SceneNode, WorldMatrix, WorldTransform,
};
pub use schedule::{Schedule, SystemStage};
pub use zircon_framework::render::{
    aspect_ratio_from_viewport_size, default_viewport_aspect_ratio, DisplayMode,
    FallbackSkyboxKind, GridMode, ProjectionMode, SceneViewportExtractRequest,
    SceneViewportSettings, SceneViewportTool, TransformSpace, ViewOrientation, ViewportCameraSnapshot,
    GridOverlayExtract, HandleElementExtract, HandleOverlayExtract, OverlayAxis,
    OverlayBillboardIcon, OverlayLineSegment, OverlayPickShape, OverlayWireShape,
    PreviewEnvironmentExtract, RenderBakedLightingExtract, RenderBloomSettings,
    RenderColorGradingSettings, RenderDirectionalLightSnapshot, RenderExtractPacket,
    RenderHybridGiExtract, RenderHybridGiProbe, RenderHybridGiTraceRegion, RenderMeshSnapshot,
    RenderOverlayExtract, RenderParticleSpriteSnapshot, RenderReflectionProbeSnapshot,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderVirtualGeometryCluster,
    RenderVirtualGeometryExtract, RenderVirtualGeometryPage, SceneGizmoKind,
    SceneGizmoOverlayExtract, SceneViewportRenderPacket, SelectionAnchorExtract,
    SelectionHighlightExtract, ViewportIconId,
};
pub use zircon_framework::scene::Mobility;
