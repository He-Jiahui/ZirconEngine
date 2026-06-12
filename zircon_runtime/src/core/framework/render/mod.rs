mod advanced;
mod anti_alias;
mod backend_types;
mod camera;
mod camera_ordering;
mod capture;
mod core_pipeline;
mod frame_extract;
mod frame_phase_queue_summary;
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
mod relevance;
mod scene_extract;
mod shader;
mod shadow;
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
    FrameHistoryHandle, FrameHistoryInvalidationReason, FrameHistoryStatus, GraphicsDebuggerStatus,
    MotionVectorCameraStatus, RenderCameraTargetGraphImportReport,
    RenderCameraTargetGraphImportStatus, RenderCameraTargetResolutionReport,
    RenderCameraTargetWritebackReport, RenderCameraTargetWritebackStatus, RenderCapabilityClass,
    RenderCapabilityClassReport, RenderCapabilityKind, RenderCapabilityMismatchDetail,
    RenderCapabilitySummary, RenderCommand, RenderFeatureQualitySettings, RenderGpuSceneUploadPath,
    RenderGraphExecutionCoverageReport, RenderGraphExecutionResourceReport,
    RenderGraphStageExecutionReport, RenderGraphTransientPoolReport, RenderHistoryCopyReport,
    RenderHybridGiPayloadSource, RenderPipelineHandle, RenderQualityProfile, RenderQuery,
    RenderQueueCapability, RenderStats, RenderViewportDescriptor, RenderViewportHandle,
    RenderVirtualGeometryPayloadSource, RenderingBackendInfo,
};
pub use camera::{
    aspect_ratio_from_viewport_size, default_viewport_aspect_ratio, DisplayMode,
    FallbackSkyboxKind, ProjectionMode, RenderCameraClearColor, RenderCameraTarget,
    RenderCameraTargetKind, RenderDynamicResolutionSettings, RenderLayer, RenderLayerSet,
    RenderViewportRect, SceneViewportExtractRequest, ViewportCameraSnapshot,
    ViewportRenderSettings, DEFAULT_CAMERA_EXPOSURE_EV100, DEFAULT_CAMERA_MSAA_SAMPLES,
    DEFAULT_RENDER_LAYER, DEFAULT_RENDER_LAYER_MASK,
};
pub use camera_ordering::{
    sort_render_cameras, RenderCameraOrderAmbiguity, RenderCameraOrderInput,
    RenderCameraOrderReport, RenderCameraTargetOrderKey, SortedRenderCamera,
};
pub use capture::{CapturedFrame, RenderCaptureReport, RenderCaptureSource};
pub use core_pipeline::{
    build_mesh_phase_queue, build_sprite_phase_queue, packed_sort_key_u64, CorePipelineKind,
    MeshPhaseInput, RenderPhase, RenderPhaseItem, RenderPhaseMeshSource, RenderPhaseQueue,
    RenderPhaseQueueOrderingKey, RenderPhaseQueueSummary, RenderPhaseQueueSummaryPhaseCount,
    RenderPhaseQueueSummaryPhaseOrderSpan, RenderPhaseSortComponents, RenderPhaseSortDecision,
    RenderPhaseSortDecisionField, RenderPhaseSortKey, RenderPhaseSortKeyBreakdown,
    SpritePhaseInput, RENDER_PHASES_BY_QUEUE_ORDER,
};
pub use frame_extract::{
    DebugOverlayExtract, GeometryExtract, GeometryPhaseInput, LightingExtract, ParticleExtract,
    PostProcessExtract, RenderExtractContext, RenderExtractProducer, RenderFrameExtract,
    RenderParticleGpuFrameExtract, RenderSkeletalPoseExtract, RenderViewExtract,
    RenderWorldSnapshotHandle, SpritePhaseExtractInput, StaticMeshBatchExtract, VisibilityInput,
    VisibilityRenderableInput,
};
pub use frame_phase_queue_summary::{
    RenderFramePhaseQueueSummary, RenderFramePhaseQueueSummaryPhaseCount,
    RenderFramePhaseQueueSummaryPhaseOrderSpan,
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
    BASIC_SCENE_UNIFORM_DIRECTIONAL_LIGHT_LIMIT, BASIC_SCENE_UNIFORM_POINT_LIGHT_LIMIT,
};
pub use material::{
    ColorMaterialDescriptor, RenderMaterialAlphaMode, RenderMaterialDependencySet,
    RenderMaterialDiagnosticSource, RenderMaterialFallbackPolicy, RenderMaterialFallbackReason,
    RenderMaterialFallbackUsage, RenderMaterialIssueState, RenderMaterialLightingModel,
    RenderMaterialLightingModelParseError, RenderMaterialManagementIssueIndex,
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
    RenderMaterialTextureTransform, RenderMaterialValidationError, StandardMaterialDescriptor,
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
    PostProcessStackDescriptor, RenderBlurSettings, RenderChromaticAberrationSettings,
    RenderColorLookupSettings, RenderColorLookupTextureLayout, RenderDepthOfFieldSettings,
    RenderDitherSettings, RenderFilmGrainSettings, RenderFogSettings, RenderMotionBlurSettings,
    RenderPostProcessEffectStackReport, RenderPostProcessEffectStackResourceStatus,
    RenderPostProcessEffectStackSettings, RenderPostProcessVolume, RenderPostProcessVolumeProfile,
    RenderPostProcessVolumeStack, RenderResolvedPostProcessSettings,
    RenderScreenSpaceReflectionSettings, RenderTonemapOperator, RenderTonemapSettings,
    RenderVignetteSettings, MAX_COLOR_LOOKUP_TEXTURE_SIZE, MIN_COLOR_LOOKUP_TEXTURE_SIZE,
};
pub use prepared_runtime_sidebands::RenderPreparedRuntimeSidebands;
pub use profile::{
    RenderProductFeature, RenderProductProfile, RenderProfileBundle, RenderProfileValidationError,
    RENDER_PROFILE_CONFIG_KEY,
};
pub use relevance::PrimitiveRelevance;
pub use scene_extract::{
    render_mesh_stable_instance_key, render_mesh_transform_revision, PreviewEnvironmentExtract,
    RenderBloomSettings, RenderColorGradingSettings, RenderExtractPacket, RenderHybridGiDebugView,
    RenderHybridGiExtract, RenderHybridGiQuality, RenderMeshLodSelection, RenderMeshSnapshot,
    RenderMeshStaticState, RenderParticleBoundsSnapshot, RenderParticleSpriteSnapshot,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderVirtualGeometryCluster,
    RenderVirtualGeometryDebugState, RenderVirtualGeometryExtract,
    RenderVirtualGeometryHierarchyNode, RenderVirtualGeometryInstance, RenderVirtualGeometryPage,
    RenderVirtualGeometryPageDependency, SceneViewportRenderPacket,
    RENDER_MESH_STABLE_KEY_MAX_PRIMITIVE_ORDINAL, RENDER_MESH_STABLE_KEY_PRIMITIVE_BITS,
};
pub use scene_extract::{RenderHybridGiProbe, RenderHybridGiTraceRegion};
pub use shader::{
    RenderShaderBindGroupLayoutDescriptor, RenderShaderBindingDescriptor,
    RenderShaderBindingResourceType, RenderShaderDefinitionValue, RenderShaderDependency,
    RenderShaderEntryPointDescriptor, RenderShaderPipelineLayoutDescriptor, RenderShaderStage,
    RenderShaderVariantKey,
};
pub use shadow::RenderShadowExecutionReport;
pub use solari::{
    SolariCapabilityRequirement, SolariDegradationReason, SolariProviderAvailability,
    SolariRuntimeDegradation, SolariRuntimeReport, SolariRuntimeStatus, SolariSettings,
};
pub use sprite::{
    RenderSpriteAnchor, RenderSpriteAtlasRegion, RenderSpriteBounds, RenderSpriteImageMode,
    RenderSpriteRect, RenderSpriteScalingMode, RenderSpriteSliceBorder, RenderSpriteSliceScaleMode,
    RenderSpriteSlicer, RenderSpriteSnapshot, SpriteExtract,
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
