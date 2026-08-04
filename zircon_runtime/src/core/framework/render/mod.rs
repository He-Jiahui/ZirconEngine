mod advanced;
mod advanced_lighting;
mod anti_alias;
mod backend_types;
mod camera;
mod camera_ordering;
mod camera_stack;
mod capture;
mod core_pipeline;
mod environment;
mod frame_extract;
mod frame_phase_queue_summary;
mod frame_profile;
mod framework;
mod framework_error;
mod highlight_set;
mod image;
mod light;
mod material;
mod mesh;
mod module_identity;
mod overlay;
mod plugin_renderer_outputs;
mod post_process;
mod prepared_runtime_sidebands;
mod profile;
mod relevance;
mod renderer_common;
mod scene_extract;
mod shader;
mod shadow;
mod solari;
mod sprite;
mod submission;
mod surface;
mod temporal_jitter;
mod view_matrix_pair;
mod virtual_geometry_debug_snapshot;
mod virtual_geometry_debug_snapshot_streams;
mod virtual_geometry_execution_draw;
mod visible_spatial_query;
mod viewport_highlight_store;

pub use advanced::{
    AdvancedProfileRuntimePlan, AdvancedProviderAvailability, AdvancedProviderReport,
    AdvancedProviderStatus, AdvancedRenderDegradation, AdvancedRenderDegradationReason,
    AdvancedRenderFeature,
};
pub use advanced_lighting::{
    burley_radial_pdf, derive_planar_reflection_camera, henyey_greenstein_phase,
    integrate_volumetric_step, oit_support, planar_oblique_near_clip_projection,
    planar_reflection_matrix, resolve_subsurface_profile_table, select_irradiance_volume,
    select_irradiance_volume_for_view, AdvancedLightingExtract, AdvancedPbrMaterialFrameUsage,
    CookieProjection, CookieWrapMode, FogVolumeData, FroxelGridParams, FroxelGridQuality,
    IrradianceVolumeData, LightCookieData, OitBufferPlan, OitCapabilityProfile, OitSettings,
    OitSupport, PlanarReflectionProbeData, PlanarReflectionQuality, PlanarReflectionUpdateState,
    PlanarUpdateMode, ScreenSpaceTransmissionSettings, StandardPbrMaterialFeatures,
    SubsurfaceProfileData, SubsurfaceProfileDiagnostic, SubsurfaceProfileTable,
    VolumetricFogSettings, VolumetricIntegrationStep, MAX_SCREEN_SPACE_TRANSMISSION_STEPS,
    OIT_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE, STANDARD_PBR_DEFAULT_CLEARCOAT_ROUGHNESS,
    STANDARD_PBR_DEFAULT_IOR, STANDARD_PBR_NO_ATTENUATION_DISTANCE,
    STANDARD_PBR_TRANSMISSION_RENDER_QUEUE, VOLUMETRIC_FOG_COMPONENT_ID,
    VOLUMETRIC_FOG_VOLUME_COMPONENT, ZR_SSS_BURLEY_SAMPLE_COUNT, ZR_SSS_MAX_PROFILES,
};
pub use anti_alias::{
    AntiAliasFallbackReason, AntiAliasFallbackReport, AntiAliasMode, AntiAliasSettings,
    TaaQualityPreset,
};
pub(crate) use backend_types::{normalize_texture_max_anisotropy, RenderGraphPassProfileMetrics};
pub use backend_types::{
    FrameHistoryHandle, FrameHistoryInvalidationReason, FrameHistoryStatus, GraphicsDebuggerStatus,
    MotionVectorCameraStatus, RenderCameraTargetGraphImportReport,
    RenderCameraTargetGraphImportStatus, RenderCameraTargetResolutionReport,
    RenderCameraTargetWritebackReport, RenderCameraTargetWritebackStatus, RenderCapabilityClass,
    RenderCapabilityClassReport, RenderCapabilityKind, RenderCapabilityMismatchDetail,
    RenderCapabilitySummary, RenderCommand, RenderDeviceDiagnostics, RenderDeviceLimitDiagnostics,
    RenderFeatureQualitySettings, RenderGpuSceneUploadPath, RenderGraphExecutionAliasRecord,
    RenderGraphExecutionAliasReport, RenderGraphExecutionCoverageReport,
    RenderGraphExecutionProfileReport, RenderGraphExecutionResourceReport,
    RenderGraphMaterializationReport, RenderGraphPassProfileRecord,
    RenderGraphStageExecutionReport, RenderGraphTransientPoolReport, RenderHistoryCopyReport,
    RenderHybridGiPayloadSource, RenderPipelineHandle, RenderQualityProfile, RenderQuery,
    RenderQueueCapability, RenderSceneVelocityReadbackReport, RenderStats,
    RenderViewportDescriptor, RenderViewportHandle, RenderVirtualGeometryPayloadSource,
    RenderingBackendInfo, DEFAULT_HALF_RES_TRANSPARENCY_DEPTH_SIGMA,
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
pub use camera_stack::{
    resolve_camera_sequence, resolve_camera_sequence_borrowed, CameraRenderDescriptor,
    CameraRenderType, CameraSequenceEntry, CameraSequenceReport, CameraSequenceViolation,
    CameraSequenceViolationReason, RenderCameraClear,
};
pub use capture::{CapturedFrame, RenderCaptureReport, RenderCaptureSource};
pub use core_pipeline::{
    build_mesh_phase_queue, build_sprite_phase_queue, packed_sort_key_u64, CorePipelineKind,
    MeshPhaseInput, RenderPhase, RenderPhaseItem, RenderPhaseMeshSource, RenderPhaseQueue,
    RenderPhaseQueueOrderingKey, RenderPhaseQueueSummary, RenderPhaseQueueSummaryPhaseCount,
    RenderPhaseQueueSummaryPhaseOrderSpan, RenderPhaseSortComponents, RenderPhaseSortDecision,
    RenderPhaseSortDecisionField, RenderPhaseSortKey, RenderPhaseSortKeyBreakdown,
    RenderQueueValue, SpritePhaseInput, RENDER_PHASES_BY_QUEUE_ORDER,
};
pub use environment::{
    append_rgb_as_rgba16f_texels, append_rgba16f_texels, build_environment_brdf_lut,
    build_source_cubemap_from_captured_faces,
    build_source_cubemap_from_captured_faces_with_quality, build_source_cubemap_from_equirect,
    build_source_cubemap_from_source_mips, build_source_cubemap_from_source_mips_with_quality,
    build_source_cubemap_irradiance_cube, build_source_cubemap_upload_artifact,
    cubemap_direction_from_scaled_uv, cubemap_face_scaled_uv_from_direction,
    cubemap_face_size_from_equirect_height, cubemap_scaled_uv_for_texel,
    cubemap_solid_angle_from_scaled_uv, cubemap_texel_direction, cubemap_texel_solid_angle,
    decode_rgb_from_rgba16f_texels, decode_rgba16f_texels, encode_rgba16f_texels,
    environment_brdf_lut_integrate, environment_brdf_lut_texel_index, equirect_uv_from_direction,
    reflection_probe_box_project_direction, reflection_probe_influence_weight,
    resolve_ibl_bake_artifact_payload, select_ibl_bake_artifact, select_reflection_probe_blend,
    source_cubemap_capture_hash, source_cubemap_environment_with_bake_artifact,
    source_cubemap_evaluate_irradiance_sh9, source_cubemap_face_mip_offset,
    source_cubemap_face_size_from_equirect_height, source_cubemap_irradiance_mip_level,
    source_cubemap_mip_chain_with_bake_artifact, source_cubemap_mip_count, source_cubemap_mip_size,
    source_cubemap_pmrem_mip_from_roughness, source_cubemap_roughness_from_pmrem_mip,
    source_cubemap_sample_count, source_cubemap_sample_irradiance_cube, CubemapFace,
    EnvironmentBrdfLutTexel, EnvironmentExtract, IblBakeArtifactBlob, IblBakeArtifactBlobCandidate,
    IblBakeArtifactBlobError, IblBakeArtifactCandidate, IblBakeArtifactContents,
    IblBakeArtifactDescriptor, IblBakeArtifactHeader, IblBakeArtifactHeaderError,
    IblBakeArtifactPayload, IblBakeArtifactPayloadError, IblBakeArtifactReadbackError,
    IblBakeArtifactReadbackSectionKind, IblBakeArtifactReadbackSections, IblBakeArtifactRequest,
    IblBakeArtifactResolvedPayload, IblBakeArtifactSelection, IblBakeArtifactSource, IblBakeKey,
    LightProbeGridData, LightmapAtlasBudget, LightmapAtlasDescriptor, LightmapAtlasFormat,
    LightmapAtlasPage, LightmapBakeOutput, LightmapBakeRequest, LightmapBakeSceneSnapshot,
    LightmapConsumeContract, LightmapContractValidationError, LightmapInstanceSlot,
    ProbeBakeTiming, ProbeInfluenceShape, ProceduralSkyParams, ReflectionProbeBlend,
    ReflectionProbeBlendEntry, ReflectionProbeData, ReflectionProbeValidationError, ShL2Rgb,
    SkyboxMode, SkyboxSettings, SourceCubemapBakeArtifactError, SourceCubemapEnvironment,
    SourceCubemapIrradianceCube, SourceCubemapIrradianceSh9, SourceCubemapMipChain,
    SourceCubemapPrefilterQuality, SourceCubemapUploadArtifact, SourceCubemapUploadKey,
    SourceCubemapUploadMip, ENVIRONMENT_BRDF_LUT_SAMPLE_COUNT, ENVIRONMENT_BRDF_LUT_SIZE,
    IBL_BAKE_ALGORITHM_VERSION, IBL_BAKE_ARTIFACT_HEADER_SIZE,
    IBL_BAKE_ARTIFACT_RGBA16F_TEXEL_SIZE_BYTES, IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES,
    LIGHTMAP_CONSUME_CONTRACT_VERSION, LIGHTMAP_SCENE_SNAPSHOT_VERSION,
    PROCEDURAL_SKY_DEFAULT_SOURCE_REVISION, RGBA16F_TEXEL_SIZE_BYTES, SH_L2_RGB_COEFFICIENT_COUNT,
    SOURCE_CUBEMAP_FACE_COUNT, SOURCE_CUBEMAP_IRRADIANCE_COEFFICIENT_COUNT,
    SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE, SOURCE_CUBEMAP_IRRADIANCE_SOURCE_FACE_SIZE,
    SOURCE_CUBEMAP_MAX_FACE_SIZE, SOURCE_CUBEMAP_MIN_FACE_SIZE, SOURCE_CUBEMAP_PMREM_FACE_SIZE,
    SOURCE_CUBEMAP_PMREM_MIP_COUNT, SOURCE_CUBEMAP_ROUGHEST_MIP,
    SOURCE_CUBEMAP_ROUGHNESS_MIP_SCALE,
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
pub use frame_profile::{
    RenderBudgetKey, RenderFrameBudget, RenderFrameProfile, RenderPassProfileEntry,
    RenderPassPipelineStatistics, RenderSubsystemProfileEntry,
};
pub use highlight_set::{HighlightRenderAttributes, HighlightSet};
pub use framework::RenderFramework;
pub use framework_error::RenderFrameworkError;
pub use image::{
    default_color_space_for_texture_usage, default_compression_for_texture_usage,
    default_mip_filter_for_texture_usage, validate_texture_metadata, RenderImageAssetUsage,
    RenderImageColorSpace, RenderImageDescriptor, RenderImageDimension, RenderImageFallbackKind,
    RenderImageUsage, RenderSamplerAddressMode, RenderSamplerDescriptor, RenderSamplerFilter,
    TextureCompressionTarget, TextureMetadata, TextureMetadataDiagnostic,
    TextureMetadataDiagnosticSeverity, TextureMipFilter, TextureMipPolicy, TextureNormalConvention,
    TextureUsageHint,
};
pub use light::{
    GpuLightData, GpuLightType, LightShadowSettings, RenderAmbientLightSnapshot,
    RenderBakedLightingExtract, RenderDirectionalLightSnapshot, RenderLightFamilyReadiness,
    RenderLightReadinessReport, RenderPointLightSnapshot, RenderRectLightSnapshot,
    RenderSpotLightSnapshot, ShadowPcfQuality, ShadowResolutionTier, GPU_LIGHT_DATA_STRIDE,
    SHADOW_SLOT_NONE,
};
pub use material::{
    ColorMaterialDescriptor, GBufferChannelMask, MaterialPropertyOverrideBlock,
    RenderMaterialAlphaMode, RenderMaterialDependencySet, RenderMaterialDiagnosticSource,
    RenderMaterialFallbackPolicy, RenderMaterialFallbackReason, RenderMaterialFallbackUsage,
    RenderMaterialIssueState, RenderMaterialLightingModel, RenderMaterialLightingModelParseError,
    RenderMaterialManagementIssueIndex, RenderMaterialManagementIssueKind,
    RenderMaterialManagementIssueView, RenderMaterialManagementOverview,
    RenderMaterialManagementOverviewRecord, RenderMaterialManagementPageInfo,
    RenderMaterialManagementPageRequest, RenderMaterialManagementPageWindow,
    RenderMaterialManagementQuery, RenderMaterialManagementQueryControls,
    RenderMaterialManagementQueryFacet, RenderMaterialManagementQueryFacetKind,
    RenderMaterialManagementQueryFacets, RenderMaterialManagementQueryFilter,
    RenderMaterialManagementQueryFilterKind, RenderMaterialManagementQueryResult,
    RenderMaterialManagementQueryResultActions, RenderMaterialManagementQueryResultState,
    RenderMaterialManagementQueryResultStateKind, RenderMaterialManagementQuerySelection,
    RenderMaterialManagementQueryState, RenderMaterialManagementRecord,
    RenderMaterialManagementRecordSet, RenderMaterialManagementRecordSummary,
    RenderMaterialManagementSelection, RenderMaterialManagementSnapshot,
    RenderMaterialManagementSortDirection, RenderMaterialManagementSortKey,
    RenderMaterialManagementSortOrder, RenderMaterialManagementStatusIndex,
    RenderMaterialManagementStatusView, RenderMaterialPreparedState,
    RenderMaterialPropertyUniformField, RenderMaterialPropertyUniformPayload,
    RenderMaterialPropertyUniformSummary, RenderMaterialPropertyUniformUnsupported,
    RenderMaterialPropertyUniformUnsupportedReason, RenderMaterialPropertyValue,
    RenderMaterialPropertyValueState, RenderMaterialPropertyValueSummary,
    RenderMaterialReadinessDiagnostic, RenderMaterialReadinessReport,
    RenderMaterialReadinessStatus, RenderMaterialReadinessSummary, RenderMaterialTextureDimension,
    RenderMaterialTextureSlotFallback, RenderMaterialTextureSlotFallbackReason,
    RenderMaterialTextureSlotState, RenderMaterialTextureSlotSummary,
    RenderMaterialTextureTransform, RenderMaterialValidationError, ShadingModelDescriptor,
    ShadingModelId, ShadingModelRegistrationError, StandardMaterialDescriptor,
    SHADING_MODEL_GBUFFER_ALPHA_SCALE, SHADING_MODEL_ID_BLINN_PHONG, SHADING_MODEL_ID_STANDARD_PBR,
    SHADING_MODEL_ID_UNLIT, SHADING_MODEL_PLUGIN_ID_START, STANDARD_MATERIAL_MIN_ROUGHNESS,
};
pub use mesh::{RenderMeshBounds, RenderMeshDescriptor, RenderMeshKind, RenderMeshTopology};
pub use module_identity::GRAPHICS_MODULE_NAME;
pub use overlay::{
    GridOverlayExtract, HandleElementExtract, HandleOverlayExtract, OverlayAxis,
    OverlayBillboardIcon, OverlayLineSegment, OverlayPickShape, OverlayWireShape,
    RenderOverlayExtract, SceneGizmoKind, SceneGizmoOverlayExtract, SelectionAnchorExtract,
    SelectionHighlightExtract, ViewportIconId,
};
pub use viewport_highlight_store::{ViewportHighlightSet, ViewportHighlightStore};
pub use plugin_renderer_outputs::{
    RenderHybridGiCacheEntryRecord, RenderHybridGiReadbackOutputs,
    RenderHybridGiScenePrepareReadbackOutputs, RenderHybridGiScenePrepareSample,
    RenderHybridGiSurfaceCachePageRecord, RenderHybridGiTraceTileRecord,
    RenderHybridGiVoxelCellDominantNodeRecord, RenderHybridGiVoxelCellRecord,
    RenderHybridGiVoxelCellSampleRecord, RenderHybridGiVoxelClipmapRecord,
    RenderHybridGiVoxelOccupancyMaskRecord, RenderParticleGpuReadbackOutputs,
    RenderPluginRendererOutputs, RenderVirtualGeometryNodeClusterCullReadbackOutputs,
    RenderVirtualGeometryPageAssignmentRecord, RenderVirtualGeometryPageReplacementRecord,
    RenderVirtualGeometryReadbackOutputs,
};
pub use post_process::{
    interp_bool, interp_discrete, interp_float_lerp, interp_vec3_lerp, PostProcessEffectKind,
    PostProcessEffectSettings, PostProcessGraphResourceNames, PostProcessGraphValidationError,
    PostProcessPassGraph, PostProcessPassNode, PostProcessStackDescriptor,
    PostProcessVolumeExtract, RenderBlurSettings, RenderChromaticAberrationSettings,
    RenderColorLookupSettings, RenderColorLookupTextureLayout, RenderColorLutReadbackReference,
    RenderColorLutReadbackReport, RenderDepthOfFieldSettings, RenderDitherSettings,
    RenderExposureMode, RenderExposureReadbackReport, RenderExposureSettings,
    RenderFilmGrainSettings, RenderFogSettings, RenderMotionBlurSettings, RenderOutputTransfer,
    RenderPostProcessEffectStackReport, RenderPostProcessEffectStackResourceStatus,
    RenderPostProcessEffectStackSettings, RenderPostProcessTextureFormat,
    RenderPostProcessVolumeProfile, RenderResolvedPostProcessSettings,
    RenderScreenSpaceReflectionSettings, RenderTonemapOperator, RenderTonemapSettings,
    RenderVignetteSettings, ResolvedPostProcessStack, VolumeComponentApplyError,
    VolumeComponentApplyFn, VolumeComponentDescriptor, VolumeComponentOverride,
    VolumeComponentReadFn, VolumeComponentRegistry, VolumeEvaluationError, VolumeEvaluationRequest,
    VolumeEvaluator, VolumeParamInterpFn, VolumeParamSchema, VolumeParamType, VolumeParamValue,
    VolumeRegistryError, VolumeShapeExtract, BUILTIN_POST_PROCESS_VOLUME_COMPONENTS,
    COLOR_LUT_FORMAT, COLOR_LUT_IDENTITY_EPSILON_MICRO, COLOR_LUT_SIZE_DEFAULT,
    COLOR_LUT_SIZE_HIGH_QUALITY, EXPOSURE_BUFFER_WORD_COUNT, EXPOSURE_HISTOGRAM_BIN_COUNT,
    EXPOSURE_READBACK_EXPECTED_BYTE_LEN, INTERMEDIATE_HDR_FORMAT_DEFAULT,
    INTERMEDIATE_HDR_FORMAT_HIGH_QUALITY, MAX_COLOR_LOOKUP_TEXTURE_SIZE,
    MIN_COLOR_LOOKUP_TEXTURE_SIZE, OUTPUT_TRANSFER_DEFAULT, TONEMAPPED_SDR_FORMAT,
};
pub use prepared_runtime_sidebands::{
    RenderHybridGiCompositePolicy, RenderHybridGiPreparedFrame, RenderHybridGiPreparedProbe,
    RenderHybridGiPreparedProbeRtLighting, RenderHybridGiPreparedProbeSceneData,
    RenderHybridGiPreparedTraceRegionSceneData, RenderHybridGiPreparedUpdateRequest,
    RenderPreparedRuntimeSidebands, HYBRID_GI_SOURCE_BAKED_BASELINE,
    HYBRID_GI_SOURCE_DYNAMIC_DELTA, HYBRID_GI_SOURCE_FULL_DYNAMIC,
};
pub use profile::{
    RenderProductFeature, RenderProductProfile, RenderProfileBundle, RenderProfileValidationError,
    RENDER_PROFILE_CONFIG_KEY,
};
pub use relevance::PrimitiveRelevance;
pub use renderer_common::{
    CastShadowsMode, LodGroupId, MaterialOverrideSet, MotionVectorMode, RendererCommon,
};
pub use scene_extract::{
    render_mesh_stable_instance_key, render_mesh_transform_revision, PreviewEnvironmentExtract,
    RenderBloomSettings, RenderColorGradingSettings, RenderExtractPacket, RenderHybridGiDebugView,
    RenderHybridGiExtract, RenderHybridGiFallbackReason, RenderHybridGiMode, RenderHybridGiProfile,
    RenderHybridGiQuality, RenderHybridGiResolvedSettings, RenderMeshLodSelection,
    RenderMeshSnapshot, RenderMeshStaticState, RenderParticleBillboardBasisSnapshot,
    RenderParticleBoundsSnapshot, RenderParticlePreviousSpriteSnapshot,
    RenderParticleSpriteSnapshot, RenderSceneGeometryExtract, RenderSceneSnapshot,
    RenderVirtualGeometryCluster, RenderVirtualGeometryDebugState, RenderVirtualGeometryExtract,
    RenderVirtualGeometryHierarchyNode, RenderVirtualGeometryInstance, RenderVirtualGeometryPage,
    RenderVirtualGeometryPageDependency, SceneViewportRenderPacket,
    RENDER_MESH_STABLE_KEY_MAX_PRIMITIVE_ORDINAL, RENDER_MESH_STABLE_KEY_PRIMITIVE_BITS,
};
pub use shader::{
    builtin_geometry_source_descriptor, builtin_geometry_source_descriptors,
    derive_shader_import_path, is_builtin_shader_module_token, is_generated_shader_module_token,
    shader_ide_generated_material_stub_relative_path, shader_ide_module_stub_relative_path,
    shader_ide_preview_relative_path, shader_ide_preview_segments_relative_path,
    shader_ide_relative_path_string, shader_project_namespace_from_name,
    strip_wgsl_include_directives, wgsl_include_paths, ComputeDispatchBuilder, ComputeDispatchPlan,
    ComputeKernelRef, ComputePipelineCacheKey, FullscreenPassBuilder, FullscreenPassPlan,
    FullscreenPipelineCacheKey, FullscreenShaderRef, GeometrySourceBindingKind,
    GeometrySourceBindingRequirement, GeometrySourceDescriptor, GeometrySourceId,
    GeometrySourceVertexAttribute, MaterialOptionKind, MaterialOptionRef, MaterialOptionTable,
    MaterialPropertyKind, MaterialPropertyLayout, MaterialPropertySlotRef,
    MaterialTextureBindingRef, PropertyScalarClass, RenderShaderBindGroupLayoutDescriptor,
    RenderShaderBindingDescriptor, RenderShaderBindingResourceType, RenderShaderDefinitionValue,
    RenderShaderDependency, RenderShaderEntryPointDescriptor, RenderShaderPipelineLayoutDescriptor,
    RenderShaderStage, RenderShaderVariantKey, ShaderAbiBinding, ShaderAssetKind, ShaderBlendMode,
    ShaderCullMode, ShaderDepthCompare, ShaderDispatchBuildDiagnostic, ShaderDispatchExtent,
    ShaderFeatureBits, ShaderIdeModuleMap, ShaderIdeModuleMapEntry, ShaderIdeModuleSource,
    ShaderIdePreviewMap, ShaderIdePreviewSegment, ShaderIdePreviewVariant,
    ShaderImportPathDerivation, ShaderImportPathDerivationError, ShaderNamedResourceBinding,
    ShaderParameterValue, ShaderPassType, ShaderPipelineDiagnostic, ShaderPipelineDiagnosticStage,
    ShaderPipelinePrewarmState, ShaderQualityTier, ShaderQueueDescriptor, ShaderQueueSegment,
    ShaderRenderStateDescriptor, ShaderResourceAccess, ShaderResourceDescriptor,
    ShaderResourceKind, ShaderVariantKey, ShaderVariantMissReport,
    ShaderVariantPrewarmDimensionCount, ShaderVariantPrewarmDimensionSummary,
    ShaderVariantPrewarmExecutionBudget, ShaderVariantPrewarmExecutionBudgetError,
    ShaderVariantPrewarmExecutionBudgetSummary, ShaderVariantPrewarmFailure,
    ShaderVariantPrewarmManifest, ShaderVariantPrewarmManifestIntegrityError,
    ShaderVariantPrewarmReport, ShaderVariantPrewarmRequest,
    ShaderVariantPrewarmSource, ShaderVariantPrewarmSourceId,
    ShaderVariantPrewarmSourceProvenanceEntry, ShaderVariantPrewarmSourceProvenanceSummary,
    ShaderVariantPrewarmWgpuModuleValidationSummary,
    ShaderVariantPrewarmWgpuPipelineValidationSummary, ShaderVariantPrewarmWrittenVariant,
    ShaderVariantRuntimeDimensionCount, ShaderVariantRuntimeDimensionSummary,
    COMPUTE_SHADER_FIRST_RESOURCE_BINDING, COMPUTE_SHADER_PARAMS_BINDING,
    COMPUTE_SHADER_RESOURCE_GROUP, FULLSCREEN_FIRST_PASS_INPUT_BINDING, FULLSCREEN_FRAME_GROUP,
    FULLSCREEN_PARAMS_BINDING, FULLSCREEN_PASS_INPUT_GROUP, FULLSCREEN_TRIANGLE_VERTEX_ENTRY,
    GENERATED_MATERIAL_MODULE_IMPORT_PATH, GEOMETRY_SOURCE_ID_MORPHED_MESH,
    GEOMETRY_SOURCE_ID_SKINNED_MESH, GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH,
    GEOMETRY_SOURCE_ID_STATIC_MESH, GEOMETRY_SOURCE_PLUGIN_ID_START,
    GEOMETRY_SOURCE_WGSL_INCLUDE_MORPHED_MESH, GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MESH,
    GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MORPHED_MESH, GEOMETRY_SOURCE_WGSL_INCLUDE_STATIC_MESH,
    SHADER_IDE_ENV_CACHE_DIR, SHADER_IDE_ENV_SCHEMA_VERSION, SHADER_IDE_MODULE_MAP_FILE,
    SHADER_IDE_PREVIEW_DEFAULT_VARIANT, SHADER_IMPORT_PROJECT_NAMESPACE_SETTING,
    SHADER_SELF_MODULE_NAMESPACE,
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
pub use submission::RenderSubmissionConfig;
pub use surface::RenderViewportSurfaceDescriptor;
pub use temporal_jitter::{halton, TemporalJitterSample, TemporalJitterSequence};
pub use view_matrix_pair::ViewProjectionMatrixPair;
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
    RenderVirtualGeometryNodeAndClusterCullTraversalRecord, RenderVirtualGeometryPagePayload,
    RenderVirtualGeometryPagePayloadVertex, RenderVirtualGeometryPageRequestInspection,
    RenderVirtualGeometryResidentPageInspection, RenderVirtualGeometrySelectedCluster,
    RenderVirtualGeometrySelectedClusterSource, RenderVirtualGeometrySubmissionEntry,
    RenderVirtualGeometrySubmissionRecord, RenderVirtualGeometryVisBuffer64Entry,
    RenderVirtualGeometryVisBuffer64Source, RenderVirtualGeometryVisBufferMark,
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
pub use visible_spatial_query::{
    RenderSpatialBounds, RenderSpatialRay, RenderVisibleSpatialQuery,
    RenderVisibleSpatialQueryResult, RenderVisibleSpatialQuerySnapshot,
    RenderVisibleSpatialQuerySnapshotId, RenderVisibleSpatialQueryStats,
    RenderVisibleSpatialQueryView,
};

pub trait RenderingManager: Send + Sync {
    fn backend_info(&self) -> RenderingBackendInfo;
}
