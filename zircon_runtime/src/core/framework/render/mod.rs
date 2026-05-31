mod advanced;
mod anti_alias;
mod backend_types;
mod camera;
mod camera_ordering;
mod core_pipeline;
mod frame_extract;
mod framework;
mod framework_error;
mod image;
mod light;
mod material;
mod mesh;
mod overlay;
mod plugin_renderer_outputs;
mod post_process;
mod prepared_runtime_sidebands;
mod profile;
mod scene_extract;
mod shader;
mod solari;
mod sprite;
mod surface;
mod virtual_geometry_debug_snapshot;
mod virtual_geometry_debug_snapshot_streams;
mod virtual_geometry_execution_draw;

pub use advanced::{
    AdvancedProfileRuntimePlan, AdvancedProviderAvailability, AdvancedProviderReport,
    AdvancedProviderStatus, AdvancedRenderDegradation, AdvancedRenderDegradationReason,
    AdvancedRenderFeature,
};
pub use anti_alias::{
    AntiAliasFallbackReason, AntiAliasFallbackReport, AntiAliasMode, AntiAliasSettings,
};
pub use backend_types::{
    CapturedFrame, FrameHistoryHandle, GraphicsDebuggerStatus, RenderCapabilityClass,
    RenderCapabilityClassReport, RenderCapabilityKind, RenderCapabilityMismatchDetail,
    RenderCapabilitySummary, RenderCommand, RenderFeatureQualitySettings,
    RenderHybridGiPayloadSource, RenderPipelineHandle, RenderQualityProfile, RenderQuery,
    RenderQueueCapability, RenderStats, RenderViewportDescriptor, RenderViewportHandle,
    RenderVirtualGeometryPayloadSource, RenderingBackendInfo,
};
pub use camera::{
    aspect_ratio_from_viewport_size, default_viewport_aspect_ratio, DisplayMode,
    FallbackSkyboxKind, ProjectionMode, RenderCameraClearColor, RenderCameraTarget, RenderLayer,
    RenderLayerSet, RenderViewportRect, SceneViewportExtractRequest, ViewportCameraSnapshot,
    ViewportRenderSettings, DEFAULT_CAMERA_EXPOSURE_EV100, DEFAULT_CAMERA_MSAA_SAMPLES,
    DEFAULT_RENDER_LAYER, DEFAULT_RENDER_LAYER_MASK,
};
pub use camera_ordering::{
    sort_render_cameras, RenderCameraOrderAmbiguity, RenderCameraOrderInput,
    RenderCameraOrderReport, RenderCameraTargetOrderKey, SortedRenderCamera,
};
pub use core_pipeline::{
    build_mesh_phase_queue, build_sprite_phase_queue, CorePipelineKind, MeshPhaseInput,
    RenderPhase, RenderPhaseItem, RenderPhaseMeshSource, RenderPhaseQueue, RenderPhaseSortKey,
    SpritePhaseInput,
};
pub use frame_extract::{
    DebugOverlayExtract, GeometryExtract, GeometryPhaseInput, LightingExtract, ParticleExtract,
    PostProcessExtract, RenderExtractContext, RenderExtractProducer, RenderFrameExtract,
    RenderParticleGpuFrameExtract, RenderSkeletalPoseExtract, RenderViewExtract,
    RenderWorldSnapshotHandle, SpritePhaseExtractInput, VisibilityInput, VisibilityRenderableInput,
};
pub use framework::RenderFramework;
pub use framework_error::RenderFrameworkError;
pub use image::{
    RenderImageAssetUsage, RenderImageColorSpace, RenderImageDescriptor, RenderImageDimension,
    RenderImageFallbackKind, RenderImageUsage, RenderSamplerAddressMode, RenderSamplerDescriptor,
    RenderSamplerFilter,
};
pub use light::{
    RenderAmbientLightSnapshot, RenderBakedLightingExtract, RenderDirectionalLightSnapshot,
    RenderLightFamilyReadiness, RenderLightReadinessReport, RenderPointLightSnapshot,
    RenderRectLightSnapshot, RenderReflectionProbeSnapshot, RenderSpotLightSnapshot,
    BASIC_SCENE_UNIFORM_DIRECTIONAL_LIGHT_LIMIT,
};
pub use material::{
    ColorMaterialDescriptor, RenderMaterialAlphaMode, RenderMaterialDependencySet,
    RenderMaterialDiagnosticSource, RenderMaterialFallbackPolicy, RenderMaterialFallbackReason,
    RenderMaterialFallbackUsage, RenderMaterialIssueState, RenderMaterialManagementIssueIndex,
    RenderMaterialManagementIssueKind, RenderMaterialManagementIssueView,
    RenderMaterialManagementOverview, RenderMaterialManagementOverviewRecord,
    RenderMaterialManagementPageInfo, RenderMaterialManagementPageRequest,
    RenderMaterialManagementPageWindow, RenderMaterialManagementQuery,
    RenderMaterialManagementQueryControls, RenderMaterialManagementQueryFacet,
    RenderMaterialManagementQueryFacetKind, RenderMaterialManagementQueryFacets,
    RenderMaterialManagementQueryFilter, RenderMaterialManagementQueryFilterKind,
    RenderMaterialManagementQueryResult, RenderMaterialManagementQueryResultActions,
    RenderMaterialManagementQueryResultState, RenderMaterialManagementQueryResultStateKind,
    RenderMaterialManagementQuerySelection, RenderMaterialManagementQueryState,
    RenderMaterialManagementRecord, RenderMaterialManagementRecordSet,
    RenderMaterialManagementRecordSummary, RenderMaterialManagementSelection,
    RenderMaterialManagementSnapshot, RenderMaterialManagementSortDirection,
    RenderMaterialManagementSortKey, RenderMaterialManagementSortOrder,
    RenderMaterialManagementStatusIndex, RenderMaterialManagementStatusView,
    RenderMaterialPreparedState, RenderMaterialPropertyUniformField,
    RenderMaterialPropertyUniformPayload, RenderMaterialPropertyUniformSummary,
    RenderMaterialPropertyUniformUnsupported, RenderMaterialPropertyUniformUnsupportedReason,
    RenderMaterialPropertyValue, RenderMaterialPropertyValueState,
    RenderMaterialPropertyValueSummary, RenderMaterialReadinessDiagnostic,
    RenderMaterialReadinessReport, RenderMaterialReadinessStatus, RenderMaterialReadinessSummary,
    RenderMaterialTextureSlotFallback, RenderMaterialTextureSlotFallbackReason,
    RenderMaterialTextureSlotState, RenderMaterialTextureSlotSummary,
    RenderMaterialValidationError, StandardMaterialDescriptor,
};
pub use mesh::{RenderMeshBounds, RenderMeshDescriptor, RenderMeshKind, RenderMeshTopology};
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
pub use post_process::{
    PostProcessEffectKind, PostProcessEffectSettings, PostProcessGraphResourceNames,
    PostProcessGraphValidationError, PostProcessPassGraph, PostProcessPassNode,
    PostProcessStackDescriptor,
};
pub use prepared_runtime_sidebands::RenderPreparedRuntimeSidebands;
pub use profile::{
    RenderProductFeature, RenderProductProfile, RenderProfileBundle, RenderProfileValidationError,
    RENDER_PROFILE_CONFIG_KEY,
};
pub use scene_extract::{
    PreviewEnvironmentExtract, RenderBloomSettings, RenderColorGradingSettings,
    RenderExtractPacket, RenderHybridGiDebugView, RenderHybridGiExtract, RenderHybridGiQuality,
    RenderMeshSnapshot, RenderParticleBoundsSnapshot, RenderParticleSpriteSnapshot,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderVirtualGeometryCluster,
    RenderVirtualGeometryDebugState, RenderVirtualGeometryExtract,
    RenderVirtualGeometryHierarchyNode, RenderVirtualGeometryInstance, RenderVirtualGeometryPage,
    RenderVirtualGeometryPageDependency, SceneViewportRenderPacket,
};
pub use scene_extract::{RenderHybridGiProbe, RenderHybridGiTraceRegion};
pub use shader::{
    RenderShaderBindGroupLayoutDescriptor, RenderShaderBindingDescriptor,
    RenderShaderBindingResourceType, RenderShaderDefinitionValue, RenderShaderDependency,
    RenderShaderEntryPointDescriptor, RenderShaderPipelineLayoutDescriptor, RenderShaderStage,
    RenderShaderVariantKey,
};
pub use solari::{
    SolariCapabilityRequirement, SolariDegradationReason, SolariProviderAvailability,
    SolariRuntimeDegradation, SolariRuntimeReport, SolariRuntimeStatus, SolariSettings,
};
pub use sprite::{
    RenderSpriteAnchor, RenderSpriteAtlasRegion, RenderSpriteBounds, RenderSpriteRect,
    RenderSpriteSnapshot, SpriteExtract,
};
pub use surface::{RenderNativeSurfaceTarget, RenderViewportSurfaceDescriptor};
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
