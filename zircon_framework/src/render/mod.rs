mod backend_types;
mod camera;
mod frame_extract;
mod framework;
mod framework_error;
mod overlay;
mod scene_extract;

pub use backend_types::{
    CapturedFrame, FrameHistoryHandle, RenderCapabilitySummary, RenderCommand,
    RenderFeatureQualitySettings, RenderPipelineHandle, RenderQualityProfile, RenderQuery,
    RenderQueueCapability, RenderStats, RenderViewportDescriptor, RenderViewportHandle,
    RenderingBackendInfo,
};
pub use camera::{
    aspect_ratio_from_viewport_size, default_viewport_aspect_ratio, DisplayMode,
    FallbackSkyboxKind, GridMode, ProjectionMode, SceneViewportExtractRequest,
    SceneViewportSettings, SceneViewportTool, TransformSpace, ViewOrientation,
    ViewportCameraSnapshot,
};
pub use frame_extract::{
    DebugOverlayExtract, GeometryExtract, LightingExtract, ParticleExtract, PostProcessExtract,
    RenderExtractContext, RenderExtractProducer, RenderFrameExtract, RenderViewExtract,
    RenderWorldSnapshotHandle, VisibilityInput, VisibilityRenderableInput,
};
pub use framework::RenderFramework;
pub use framework_error::RenderFrameworkError;
pub use overlay::{
    GridOverlayExtract, HandleElementExtract, HandleOverlayExtract, OverlayAxis,
    OverlayBillboardIcon, OverlayLineSegment, OverlayPickShape, OverlayWireShape,
    RenderOverlayExtract, SceneGizmoKind, SceneGizmoOverlayExtract, SelectionAnchorExtract,
    SelectionHighlightExtract, ViewportIconId,
};
pub use scene_extract::{
    PreviewEnvironmentExtract, RenderBakedLightingExtract, RenderDirectionalLightSnapshot,
    RenderBloomSettings, RenderColorGradingSettings, RenderExtractPacket, RenderHybridGiExtract,
    RenderHybridGiProbe, RenderHybridGiTraceRegion, RenderMeshSnapshot,
    RenderParticleSpriteSnapshot, RenderReflectionProbeSnapshot, RenderSceneGeometryExtract,
    RenderSceneSnapshot, RenderVirtualGeometryCluster, RenderVirtualGeometryExtract,
    RenderVirtualGeometryPage, SceneViewportRenderPacket,
};

pub trait RenderingManager: Send + Sync {
    fn backend_info(&self) -> RenderingBackendInfo;
}
