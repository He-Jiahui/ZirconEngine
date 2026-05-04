mod backend_types;
mod camera;
mod frame_extract;
mod framework;
mod framework_error;
mod overlay;
mod plugin_renderer_outputs;
mod prepared_runtime_sidebands;
mod scene_extract;
mod virtual_geometry_debug_snapshot;
mod virtual_geometry_debug_snapshot_streams;
mod virtual_geometry_execution_draw;

pub use backend_types::{
    CapturedFrame, FrameHistoryHandle, RenderCapabilityKind, RenderCapabilityMismatchDetail,
    RenderCapabilitySummary, RenderCommand, RenderFeatureQualitySettings, RenderPipelineHandle,
    RenderQualityProfile, RenderQuery, RenderQueueCapability, RenderStats,
    RenderViewportDescriptor, RenderViewportHandle, RenderingBackendInfo,
};
pub use camera::{
    aspect_ratio_from_viewport_size, default_viewport_aspect_ratio, DisplayMode,
    FallbackSkyboxKind, ProjectionMode, SceneViewportExtractRequest, ViewportCameraSnapshot,
    ViewportRenderSettings,
};
pub use frame_extract::{
    DebugOverlayExtract, GeometryExtract, LightingExtract, ParticleExtract, PostProcessExtract,
    RenderExtractContext, RenderExtractProducer, RenderFrameExtract, RenderParticleGpuFrameExtract,
    RenderSkeletalPoseExtract, RenderViewExtract, RenderWorldSnapshotHandle, VisibilityInput,
    VisibilityRenderableInput,
};
pub use framework::RenderFramework;
pub use framework_error::RenderFrameworkError;
pub use overlay::{
    GridOverlayExtract, HandleElementExtract, HandleOverlayExtract, OverlayAxis,
    OverlayBillboardIcon, OverlayLineSegment, OverlayPickShape, OverlayWireShape,
    RenderOverlayExtract, SceneGizmoKind, SceneGizmoOverlayExtract, SelectionAnchorExtract,
    SelectionHighlightExtract, ViewportIconId,
};
pub use plugin_renderer_outputs::{
    RenderHybridGiCacheEntryRecord, RenderHybridGiReadbackOutputs,
    RenderHybridGiScenePrepareReadbackOutputs, RenderHybridGiScenePrepareSample,
    RenderHybridGiVoxelCellDominantNodeRecord, RenderHybridGiVoxelCellRecord,
    RenderHybridGiVoxelCellSampleRecord, RenderHybridGiVoxelOccupancyMaskRecord,
    RenderParticleGpuReadbackOutputs, RenderPluginRendererOutputs,
    RenderVirtualGeometryNodeClusterCullReadbackOutputs, RenderVirtualGeometryPageAssignmentRecord,
    RenderVirtualGeometryPageReplacementRecord, RenderVirtualGeometryReadbackOutputs,
};
pub use prepared_runtime_sidebands::RenderPreparedRuntimeSidebands;
pub use scene_extract::{
    PreviewEnvironmentExtract, RenderBakedLightingExtract, RenderBloomSettings,
    RenderColorGradingSettings, RenderDirectionalLightSnapshot, RenderExtractPacket,
    RenderHybridGiDebugView, RenderHybridGiExtract, RenderHybridGiQuality, RenderMeshSnapshot,
    RenderParticleBoundsSnapshot, RenderParticleSpriteSnapshot, RenderPointLightSnapshot,
    RenderReflectionProbeSnapshot, RenderSceneGeometryExtract, RenderSceneSnapshot,
    RenderSpotLightSnapshot, RenderVirtualGeometryCluster, RenderVirtualGeometryDebugState,
    RenderVirtualGeometryExtract, RenderVirtualGeometryHierarchyNode,
    RenderVirtualGeometryInstance, RenderVirtualGeometryPage, RenderVirtualGeometryPageDependency,
    SceneViewportRenderPacket,
};
pub use scene_extract::{RenderHybridGiProbe, RenderHybridGiTraceRegion};
pub use virtual_geometry_debug_snapshot::{
    RenderVirtualGeometryBvhVisualizationInstance, RenderVirtualGeometryBvhVisualizationNode,
    RenderVirtualGeometryClusterSelectionInputSource,
    RenderVirtualGeometryCpuReferenceDepthClusterMapEntry,
    RenderVirtualGeometryCpuReferenceInstance, RenderVirtualGeometryCpuReferenceLeafCluster,
    RenderVirtualGeometryCpuReferenceMipClusterMapEntry,
    RenderVirtualGeometryCpuReferenceNodeVisit,
    RenderVirtualGeometryCpuReferencePageClusterMapEntry,
    RenderVirtualGeometryCpuReferencePageDependencyEntry,
    RenderVirtualGeometryCpuReferenceSelectedCluster, RenderVirtualGeometryCullInputSnapshot,
    RenderVirtualGeometryDebugSnapshot, RenderVirtualGeometryExecutionSegment,
    RenderVirtualGeometryExecutionState, RenderVirtualGeometryHardwareRasterizationRecord,
    RenderVirtualGeometryHardwareRasterizationSource,
    RenderVirtualGeometryNodeAndClusterCullChildWorkItem,
    RenderVirtualGeometryNodeAndClusterCullClusterWorkItem,
    RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
    RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    RenderVirtualGeometryNodeAndClusterCullInstanceSeed,
    RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem,
    RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot,
    RenderVirtualGeometryNodeAndClusterCullSource,
    RenderVirtualGeometryNodeAndClusterCullTraversalChildSource,
    RenderVirtualGeometryNodeAndClusterCullTraversalOp,
    RenderVirtualGeometryNodeAndClusterCullTraversalRecord,
    RenderVirtualGeometryPageRequestInspection, RenderVirtualGeometryResidentPageInspection,
    RenderVirtualGeometrySelectedCluster, RenderVirtualGeometrySelectedClusterSource,
    RenderVirtualGeometrySubmissionEntry, RenderVirtualGeometrySubmissionRecord,
    RenderVirtualGeometryVisBuffer64Entry, RenderVirtualGeometryVisBuffer64Source,
    RenderVirtualGeometryVisBufferMark,
};
pub use virtual_geometry_debug_snapshot_streams::{
    RenderVirtualGeometryDebugSnapshotDecodedStreams,
    RenderVirtualGeometryDebugSnapshotReadbackStreamDecodeDiagnostic,
    RenderVirtualGeometryDebugSnapshotReadbackStreamDecodeError,
    RenderVirtualGeometryDebugSnapshotReadbackStreamFootprint,
    RenderVirtualGeometryDebugSnapshotReadbackStreamReport,
    RenderVirtualGeometryDebugSnapshotReadbackStreamSection,
    RenderVirtualGeometryDebugSnapshotReadbackStreamSummary,
    RenderVirtualGeometryDebugSnapshotReadbackStreams,
    RenderVirtualGeometryNodeAndClusterCullDecodedStreams,
    RenderVirtualGeometryNodeAndClusterCullWordStreamDecodeError,
    RenderVirtualGeometryNodeAndClusterCullWordStreams,
    RenderVirtualGeometryRenderPathDecodedStreams,
    RenderVirtualGeometryRenderPathWordStreamDecodeError,
    RenderVirtualGeometryRenderPathWordStreams, RenderVirtualGeometryVisBuffer64DecodedStream,
    RenderVirtualGeometryVisBuffer64ReadbackStream,
    RenderVirtualGeometryVisBuffer64ReadbackStreamDecodeError,
};
pub use virtual_geometry_execution_draw::RenderVirtualGeometryExecutionDraw;

pub trait RenderingManager: Send + Sync {
    fn backend_info(&self) -> RenderingBackendInfo;
}
