mod backend_types;
mod camera;
mod frame_extract;
mod framework;
mod framework_error;
mod overlay;
mod scene_extract;
mod virtual_geometry_debug_snapshot;

pub use backend_types::{
    CapturedFrame, FrameHistoryHandle, RenderCapabilitySummary, RenderCommand,
    RenderFeatureQualitySettings, RenderPipelineHandle, RenderQualityProfile, RenderQuery,
    RenderQueueCapability, RenderStats, RenderViewportDescriptor, RenderViewportHandle,
    RenderingBackendInfo,
};
pub use camera::{
    aspect_ratio_from_viewport_size, default_viewport_aspect_ratio, DisplayMode,
    FallbackSkyboxKind, ProjectionMode, SceneViewportExtractRequest, ViewportCameraSnapshot,
    ViewportRenderSettings,
};
pub use frame_extract::{
    DebugOverlayExtract, GeometryExtract, LightingExtract, ParticleExtract, PostProcessExtract,
    RenderExtractContext, RenderExtractProducer, RenderFrameExtract, RenderSkeletalPoseExtract,
    RenderViewExtract, RenderWorldSnapshotHandle, VisibilityInput, VisibilityRenderableInput,
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
    PreviewEnvironmentExtract, RenderBakedLightingExtract, RenderBloomSettings,
    RenderColorGradingSettings, RenderDirectionalLightSnapshot, RenderExtractPacket,
    RenderHybridGiDebugView, RenderHybridGiExtract, RenderHybridGiQuality, RenderMeshSnapshot,
    RenderParticleSpriteSnapshot, RenderPointLightSnapshot, RenderReflectionProbeSnapshot,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderSpotLightSnapshot,
    RenderVirtualGeometryCluster, RenderVirtualGeometryDebugState, RenderVirtualGeometryExtract,
    RenderVirtualGeometryInstance, RenderVirtualGeometryPage, SceneViewportRenderPacket,
};
pub(crate) use scene_extract::{RenderHybridGiProbe, RenderHybridGiTraceRegion};
pub use virtual_geometry_debug_snapshot::{
    RenderVirtualGeometryBvhVisualizationInstance, RenderVirtualGeometryBvhVisualizationNode,
    RenderVirtualGeometryClusterSelectionInputSource,
    RenderVirtualGeometryCpuReferenceDepthClusterMapEntry,
    RenderVirtualGeometryCpuReferenceInstance, RenderVirtualGeometryCpuReferenceLeafCluster,
    RenderVirtualGeometryCpuReferenceMipClusterMapEntry,
    RenderVirtualGeometryCpuReferenceNodeVisit,
    RenderVirtualGeometryCpuReferencePageClusterMapEntry,
    RenderVirtualGeometryCpuReferenceSelectedCluster, RenderVirtualGeometryCullInputSnapshot,
    RenderVirtualGeometryDebugSnapshot, RenderVirtualGeometryExecutionSegment,
    RenderVirtualGeometryExecutionState, RenderVirtualGeometryHardwareRasterizationRecord,
    RenderVirtualGeometryHardwareRasterizationSource,
    RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
    RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    RenderVirtualGeometryNodeAndClusterCullInstanceSeed,
    RenderVirtualGeometryNodeAndClusterCullSource, RenderVirtualGeometryPageRequestInspection,
    RenderVirtualGeometryResidentPageInspection, RenderVirtualGeometrySelectedCluster,
    RenderVirtualGeometrySelectedClusterSource, RenderVirtualGeometrySubmissionEntry,
    RenderVirtualGeometrySubmissionRecord, RenderVirtualGeometryVisBuffer64Entry,
    RenderVirtualGeometryVisBuffer64Source, RenderVirtualGeometryVisBufferMark,
};

pub trait RenderingManager: Send + Sync {
    fn backend_info(&self) -> RenderingBackendInfo;
}
