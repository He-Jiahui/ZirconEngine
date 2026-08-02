//! Runtime asset subsystem: project manifests, loading, import, and pipeline runtime support.

mod module;
pub mod prelude;

pub use module::{
    ASSET_IO_DRIVER_NAME, ASSET_MANAGER_NAME, ASSET_MODULE_NAME, AssetModule,
    PROJECT_ASSET_MANAGER_NAME, RESOURCE_MANAGER_NAME, module_descriptor,
};

pub mod artifact;
pub mod assets;
pub mod facade;
mod formats;
pub mod importer;
mod load;
mod management;
pub mod migration;
pub mod pack;
pub mod pipeline;
pub mod project;
mod reference_resolution_error;
mod reference_resolver;
pub mod registry;
mod runtime_asset_path;
mod safe_project_path;
mod virtual_geometry_cook;
pub mod watch;

pub(crate) use artifact::ArtifactStore;
pub use assets::{
    AlphaMode, AssetAuthoringError, AssetAuthoringResult, CUBEMAP_FACE_COUNT, CubeLutParseError,
    CubemapAsset, CubemapAssetError, CubemapSourceLayout, DataAsset, DataAssetFormat,
    EXTERNAL_SOURCE_CUBEMAP_UPLOAD_UNSUPPORTED_REASON, ExternalSourceCubemapContainerError,
    ExternalSourceCubemapContainerInfo, ExternalSourceCubemapContainerKind,
    ExternalSourceCubemapDecodeError, FontAsset, FontAssetCmapCoverage, FontAssetCodepointRange,
    FontAssetError, FontAssetFaceMetrics, FontAssetFaceStyle, FontAssetFamilyMember,
    FontAssetLineMetrics, FontAssetMetadata, FontAssetParsedFace, FontAssetRenderStrategy,
    FontAssetResult, FontAssetSourceFormat, FontAssetVariableInstance, FontAssetVariationAxis,
    FontAssetVariationCoord, IBL_PMREM_RGBA16F_FORMAT, IBL_PMREM_RGBA16F_GPU_FORMAT,
    IblPmremTextureError, ImportedAsset, LIGHTMAP_RGBA16F_FORMAT, LIGHTMAP_RGBA16F_GPU_FORMAT,
    MESH_ATTRIBUTE_COLOR, MESH_ATTRIBUTE_JOINT_INDEX, MESH_ATTRIBUTE_JOINT_WEIGHT,
    MESH_ATTRIBUTE_NORMAL, MESH_ATTRIBUTE_POSITION, MESH_ATTRIBUTE_TANGENT, MESH_ATTRIBUTE_UV0,
    MESH_ATTRIBUTE_UV1, MaterialAsset, MaterialAssetManagementRecord,
    MaterialAssetManagementRecordSet, MaterialAssetManagementRecordSetSummary,
    MaterialAssetOverview, MaterialGraphAsset, MaterialGraphLinkAsset, MaterialGraphNodeAsset,
    MaterialGraphNodeKindAsset, MaterialGraphParameterAsset, MaterialTextureSlotValue, MeshAsset,
    MeshAssetManagementRecord, MeshAssetManagementRecordFailure, MeshAssetManagementRecordSet,
    MeshAssetManagementRecordSetSummary, MeshAssetOverview, MeshAssetUsage, MeshAttributeFormat,
    MeshAttributeSummary, MeshAttributeValues, MeshIndexFormat, MeshIndices, MeshMorphTargetAsset,
    MeshMorphTargetAttributeSummary, MeshSkinAsset, MeshValidationError, ModelAsset,
    ModelAssetManagementRecord, ModelAssetManagementRecordSet,
    ModelAssetManagementRecordSetSummary, ModelAssetOverview, ModelPrimitiveAsset,
    ModelPrimitiveOverview, PhysicsMaterialAsset, PrefabAsset, PrefabInstanceAsset,
    PrefabPropertyOverrideAsset, RGBA8_UNORM_FORMAT, RGBA8_UNORM_SRGB_FORMAT,
    SceneAmbientLightAsset, SceneAnimationGraphPlayerAsset, SceneAnimationPlayerAsset,
    SceneAnimationSequencePlayerAsset, SceneAnimationSkeletonAsset,
    SceneAnimationStateMachinePlayerAsset, SceneAsset, SceneAssetManagementRecord,
    SceneAssetManagementRecordSet, SceneAssetManagementRecordSetSummary, SceneAssetOverview,
    SceneBloomSettingsAsset, SceneCameraAsset, SceneCameraTargetAsset,
    SceneChromaticAberrationSettingsAsset, SceneColliderAsset, SceneColliderShapeAsset,
    SceneColorGradingSettingsAsset, SceneDirectionalLightAsset, SceneDitherSettingsAsset,
    SceneEntityAsset, SceneEntityManagementRecord, SceneEntityManagementRecordSet,
    SceneEntityManagementRecordSetSummary, SceneEntityOverview, SceneFilmGrainSettingsAsset,
    SceneFogSettingsAsset, SceneJointAsset, SceneJointKindAsset, SceneMeshInstanceAsset,
    SceneMeshLodLevelAsset, SceneMeshPrimitiveBindingAsset, SceneMobilityAsset,
    ScenePointLightAsset, ScenePostProcessEffectStackAsset, ScenePostProcessSettingsAsset,
    ScenePostProcessVolumeAsset, ScenePostProcessVolumeProfileAsset, SceneRectLightAsset,
    SceneRigidBodyAsset, SceneRigidBodyTypeAsset, SceneScriptBindingAsset, SceneSpotLightAsset,
    SceneTerrainAsset, SceneTileMapAsset, SceneTonemapOperatorAsset, SceneTonemapSettingsAsset,
    SceneViewportRectAsset, SceneVignetteSettingsAsset, SceneVolumetricFogSettingsAsset,
    ShaderAsset, ShaderAssetManagementRecord, ShaderAssetManagementRecordSet,
    ShaderAssetManagementRecordSetSummary, ShaderAssetReadinessSummary,
    ShaderBindGroupLayoutReadiness, ShaderBindingLayoutReadiness, ShaderDefinitionReadiness,
    ShaderDependencyAsset, ShaderEntryPointAsset, ShaderEntryPointReadiness, ShaderImportReadiness,
    ShaderImportRedirectAsset, ShaderMaterialPropertyAsset, ShaderOptionAsset,
    ShaderPipelineLayoutReadiness, ShaderReadinessReport, ShaderRuntimeSourceKind,
    ShaderRuntimeSourceReadiness, ShaderSourceFileAsset, ShaderSourceLanguage,
    ShaderTextureSlotAsset, SoundAsset, SoundAssetError, SoundAssetResult, SpriteAtlasAsset,
    SpriteAtlasEntry, SpriteAtlasPadding, SpriteAtlasRect, SpriteAtlasUvRect,
    SpriteAtlasValidationError, TerrainAsset, TerrainLayerAsset, TerrainLayerStackAsset,
    Texture2DArrayAsset, Texture2DArrayAssetError, TextureArrayLayerSource, TextureArrayLayout,
    TextureAsset, TextureAssetDescriptor, TexturePayload, TextureUploadCompressionFamily,
    TextureUploadPlan, TextureUploadReadiness, TextureUploadSupport, TileMapAsset,
    TileMapLayerAsset, TileMapProjectionAsset, TileSetAsset, TileSetTileAsset, TransformAsset,
    UiAssetDocumentError, UiAssetDocumentResult, UiIconAsset, UiIconAssetDocumentError,
    UiIconAssetDocumentResult, UiIconSource, UiIconSourceKind, UiLayoutAsset, UiStyleAsset,
    UiThemeAsset, UiThemeAssetDocumentError, UiThemeAssetDocumentResult, UiV2AssetDocumentError,
    UiV2AssetDocumentResult, UiV2ComponentAsset, UiV2StyleAsset, UiV2ViewAsset, UiWidgetAsset,
    VirtualGeometryAsset, VirtualGeometryClusterHeaderAsset, VirtualGeometryClusterPageHeaderAsset,
    VirtualGeometryDebugMetadataAsset, VirtualGeometryHierarchyNodeAsset,
    VirtualGeometryPageDependencyAsset, VirtualGeometryRootClusterRangeAsset,
    ZCUBE_SOURCE_CUBEMAP_FORMAT, ZCUBE_SOURCE_CUBEMAP_GPU_FORMAT, ZCUBE_SOURCE_CUBEMAP_HEADER_SIZE,
    ZMESH_DOCUMENT_VERSION, ZMaterialDocument, ZMaterialQueueOverride, ZMeshDocument,
    ZShaderComputeDocumentV2, ZShaderDocumentV2, ZShaderEntryPointDocument,
    ZShaderFullscreenDocumentV2, ZShaderImportDocument, ZShaderIncludeDocumentV2,
    ZShaderOptionDocument, ZShaderSurfaceDocumentV2, ZShaderTextureSlotDocument, ZShaderV2Error,
    ZShaderV2Result, ZcubeSourceCubemap, ZcubeSourceCubemapError, asset_kind_for_imported_asset,
    decode_external_source_cubemap, decode_ibl_pmrem_rgba16f_texture,
    decode_zcube_source_cubemap_texture, external_source_cubemap_container_info,
    is_external_source_cubemap_container, is_ibl_pmrem_rgba16f_texture,
    is_zcube_source_cubemap_texture, texture_asset_from_array_layers, texture_asset_from_cube_lut,
    texture_asset_from_cubemap_faces, texture_asset_from_ibl_bake_artifact_pmrem,
    texture_asset_from_lightmap_bake_output, texture_asset_from_source_cubemap_zcube,
    validate_sprite_atlas_asset, validate_wgsl_captures,
};
pub use facade::{
    Asset, AssetDependencyReadiness, AssetEvent, AssetEventKind, AssetEventReceiver,
    AssetLoadState, AssetLoadStates, AssetReadinessNode, AssetReadinessReport, Assets,
    DependencyLoadState, Handle, RecursiveDependencyLoadState,
};
pub use importer::{
    AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporter,
    AssetImporterCapabilityReport, AssetImporterCapabilityStatus, AssetImporterDescriptor,
    AssetImporterHandler, AssetImporterRegistry, AssetImporterRegistryError,
    AssetSchemaMigrationReport, AssetSchemaMigrator, DecodedTextureImage,
    DecodedTextureImageRgba32F, DiagnosticOnlyAssetImporter,
    ENVIRONMENT_IBL_FACE_SIZE_IMPORT_SETTING, ENVIRONMENT_IBL_IMPORT_SETTING,
    EnvironmentIblSourceStagingError, EnvironmentIblSourceStagingReport,
    EnvironmentIblSourceStagingStatus, FunctionAssetImporter, ImportedAssetEntry,
    NativeAssetImportCommandHost, NativeAssetImportCommandReport, NativeAssetImportCommandStatus,
    NativeAssetImportEntryMetadata, NativeAssetImportRequestMetadata,
    NativeAssetImportResponseMetadata, NativeAssetImporterHandler, StaticAssetSchemaMigrator,
    decode_texture_source_image, decode_texture_source_image_rgba32f, stage_environment_ibl_source,
    stage_external_source_cubemap_texture,
};
pub use management::{
    AssetManagementFamilyIssueBucket, AssetManagementFamilyIssueIndex,
    AssetManagementFamilyIssueView, AssetManagementFamilyKind, AssetManagementFamilyStatus,
    AssetManagementFamilyStatusIndex, AssetManagementFamilyStatusView,
    AssetManagementFamilySummary, AssetManagementOverview, AssetManagementRecordSetSummary,
    AssetManagementRecordSets,
};
pub use pipeline::manager::{
    AssetIoDriver, AssetManager, AssetPipelineInfo, AssetStatusRecord, ProjectAssetManager,
    ProjectAssetManagerAccess, ProjectInfo, asset_manager_handle, project_asset_manager_handle,
};
pub use pipeline::types::MeshVertex;
#[allow(unused_imports)]
pub(crate) use pipeline::types::{
    AssetRequest, CpuAssetPayload, CpuMeshPayload, CpuTexturePayload, MeshSource, TextureSource,
};
pub(crate) use pipeline::{types, worker_pool};
pub use project::ProjectManifest;
#[allow(unused_imports)]
pub(crate) use project::{
    AssetMetaDocument, AssetMetaEntry, AssetMetaError, AssetMetaResult, AssetSourceUnit,
    PackageAssetRegistry, PreviewState, ProjectManager, ProjectPaths,
};
pub use registry::{
    AssetRegistryDiagnostic, AssetRegistryEntry, AssetRegistryError, AssetRegistryFilter,
    AssetRegistryIndex,
};
pub use runtime_asset_path::{
    runtime_asset_path, runtime_asset_path_with_dev_asset_root, runtime_asset_root,
};
pub use virtual_geometry_cook::{
    VirtualGeometryCookConfig, VirtualGeometryCookRequest, VirtualGeometryCookSettings,
    cook_virtual_geometry_from_mesh, encode_virtual_geometry_cook_binary_dump,
    format_virtual_geometry_cook_bvh_graph_dump, format_virtual_geometry_cook_inspection_dump,
};

pub type AssetId = crate::core::resource::ResourceId;
pub type AssetKind = crate::core::resource::ResourceKind;
pub type AssetReference = crate::core::resource::AssetReference;
pub type AssetUri = crate::core::resource::ResourceLocator;
pub type AssetUuid = crate::core::resource::AssetUuid;
pub use reference_resolution_error::ReferenceResolutionError;
pub use reference_resolver::{ReferenceRepair, ReferenceRepairKind};
pub(crate) use reference_resolver::{ResolvedProjectReference, resolve_project_reference};

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) mod test_support {
    pub(crate) use super::tests::support::{
        write_checker_png, write_default_material, write_triangle_obj,
    };
}
