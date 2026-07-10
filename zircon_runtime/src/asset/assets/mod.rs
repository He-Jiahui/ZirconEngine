mod animation;
mod authoring;
mod data;
mod font;
#[cfg(feature = "text")]
mod font_source;
mod imported;
mod material;
mod mesh;
mod model;
mod navigation;
mod physics_material;
mod scene;
mod shader;
mod sound;
mod sprite_atlas;
mod texture;
mod ui;

pub use animation::{
    AnimationAssetError, AnimationAssetResult, AnimationChannelAsset, AnimationChannelKeyAsset,
    AnimationChannelValueAsset, AnimationClipAsset, AnimationClipBoneTrackAsset,
    AnimationConditionOperatorAsset, AnimationEventTrackAsset, AnimationGraphAsset,
    AnimationGraphNodeAsset, AnimationGraphParameterAsset, AnimationInterpolationAsset,
    AnimationSequenceAsset, AnimationSequenceBindingAsset, AnimationSequenceTrackAsset,
    AnimationSkeletonAsset, AnimationSkeletonBoneAsset, AnimationStateAsset,
    AnimationStateMachineAsset, AnimationStateTransitionAsset, AnimationTransitionConditionAsset,
};
pub use authoring::{
    AssetAuthoringError, AssetAuthoringResult, MaterialGraphAsset, MaterialGraphLinkAsset,
    MaterialGraphNodeAsset, MaterialGraphNodeKindAsset, MaterialGraphParameterAsset, PrefabAsset,
    PrefabInstanceAsset, PrefabPropertyOverrideAsset, TerrainAsset, TerrainLayerAsset,
    TerrainLayerStackAsset, TileMapAsset, TileMapLayerAsset, TileMapProjectionAsset, TileSetAsset,
    TileSetTileAsset,
};
pub use data::{DataAsset, DataAssetFormat};
pub use font::{
    FontAsset, FontAssetCmapCoverage, FontAssetCodepointRange, FontAssetError,
    FontAssetFaceMetrics, FontAssetFaceStyle, FontAssetFamilyMember, FontAssetLineMetrics,
    FontAssetMetadata, FontAssetParsedFace, FontAssetRenderStrategy, FontAssetResult,
    FontAssetSourceFormat, FontAssetVariableInstance, FontAssetVariationAxis,
    FontAssetVariationCoord,
};
#[cfg(feature = "text")]
pub(crate) use font_source::{
    decode_font_source, standalone_sfnt_face, DecodedFontSource, FontFaceExtractionError,
};
#[cfg(feature = "text")]
pub use font_source::{FontMetadataParseError, FontSourceDecodeError};
pub use imported::{asset_kind_for_imported_asset, ImportedAsset};
pub use material::{
    validate_wgsl_captures, AlphaMode, MaterialAsset, MaterialAssetManagementRecord,
    MaterialAssetManagementRecordSet, MaterialAssetManagementRecordSetSummary,
    MaterialAssetOverview, MaterialTextureSlotValue, ZMaterialDocument, ZMaterialQueueOverride,
};
pub use mesh::{
    MeshAsset, MeshAssetManagementRecord, MeshAssetManagementRecordFailure,
    MeshAssetManagementRecordSet, MeshAssetManagementRecordSetSummary, MeshAssetOverview,
    MeshAssetUsage, MeshAttributeFormat, MeshAttributeSummary, MeshAttributeValues,
    MeshIndexFormat, MeshIndices, MeshMorphTargetAsset, MeshMorphTargetAttributeSummary,
    MeshSkinAsset, MeshValidationError, ZMeshDocument, MESH_ATTRIBUTE_COLOR,
    MESH_ATTRIBUTE_JOINT_INDEX, MESH_ATTRIBUTE_JOINT_WEIGHT, MESH_ATTRIBUTE_NORMAL,
    MESH_ATTRIBUTE_POSITION, MESH_ATTRIBUTE_TANGENT, MESH_ATTRIBUTE_UV0, MESH_ATTRIBUTE_UV1,
    ZMESH_DOCUMENT_VERSION,
};
pub use model::{
    ModelAsset, ModelAssetManagementRecord, ModelAssetManagementRecordSet,
    ModelAssetManagementRecordSetSummary, ModelAssetOverview, ModelPrimitiveAsset,
    ModelPrimitiveOverview, VirtualGeometryAsset, VirtualGeometryClusterHeaderAsset,
    VirtualGeometryClusterPageHeaderAsset, VirtualGeometryDebugMetadataAsset,
    VirtualGeometryHierarchyNodeAsset, VirtualGeometryPageDependencyAsset,
    VirtualGeometryRootClusterRangeAsset,
};
pub use navigation::{
    NavMeshAreaCostAsset, NavMeshAsset, NavMeshGizmoTriangleAsset, NavMeshLinkAsset,
    NavMeshPolygonAsset, NavMeshTileAsset, NavigationAssetError, NavigationAssetResult,
    NavigationSettingsAsset,
};
pub use physics_material::PhysicsMaterialAsset;
pub use scene::{
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
    SceneViewportRectAsset, SceneVignetteSettingsAsset, TransformAsset,
};
pub use shader::{
    generate_material_artifact, ShaderAsset, ShaderAssetManagementRecord,
    ShaderAssetManagementRecordSet, ShaderAssetManagementRecordSetSummary,
    ShaderAssetReadinessSummary, ShaderBindGroupLayoutReadiness, ShaderBindingLayoutReadiness,
    ShaderDefinitionReadiness, ShaderDependencyAsset, ShaderEntryPointAsset,
    ShaderEntryPointReadiness, ShaderGeneratedMaterialArtifact, ShaderImportReadiness,
    ShaderImportRedirectAsset, ShaderMaterialPropertyAsset, ShaderOptionAsset,
    ShaderPipelineLayoutReadiness, ShaderReadinessReport, ShaderRuntimeSourceKind,
    ShaderRuntimeSourceReadiness, ShaderSourceFileAsset, ShaderSourceLanguage,
    ShaderTextureSlotAsset, ZShaderComputeDocumentV2, ZShaderDocumentV2, ZShaderEntryPointDocument,
    ZShaderFullscreenDocumentV2, ZShaderImportDocument, ZShaderIncludeDocumentV2,
    ZShaderOptionDocument, ZShaderSurfaceDocumentV2, ZShaderTextureSlotDocument, ZShaderV2Error,
    ZShaderV2Result,
};
pub use sound::{SoundAsset, SoundAssetError, SoundAssetResult};
pub use sprite_atlas::{
    validate_sprite_atlas_asset, SpriteAtlasAsset, SpriteAtlasEntry, SpriteAtlasPadding,
    SpriteAtlasRect, SpriteAtlasUvRect, SpriteAtlasValidationError,
};
pub use texture::{
    decode_external_source_cubemap, decode_ibl_pmrem_rgba16f_texture,
    decode_zcube_source_cubemap_texture, external_source_cubemap_container_info,
    is_external_source_cubemap_container, is_ibl_pmrem_rgba16f_texture,
    is_zcube_source_cubemap_texture, texture_asset_from_array_layers, texture_asset_from_cube_lut,
    texture_asset_from_cubemap_faces, texture_asset_from_ibl_bake_artifact_pmrem,
    texture_asset_from_source_cubemap_zcube, CubeLutParseError, CubemapAsset, CubemapAssetError,
    CubemapSourceLayout, ExternalSourceCubemapContainerError, ExternalSourceCubemapContainerInfo,
    ExternalSourceCubemapContainerKind, ExternalSourceCubemapDecodeError, IblPmremTextureError,
    Texture2DArrayAsset, Texture2DArrayAssetError, TextureArrayLayerSource, TextureArrayLayout,
    TextureAsset, TextureAssetDescriptor, TextureDescriptorError, TextureDescriptorResult,
    TexturePayload, TextureUploadCompressionFamily, TextureUploadPlan, TextureUploadReadiness,
    TextureUploadSupport, ZcubeSourceCubemap, ZcubeSourceCubemapError, CUBEMAP_FACE_COUNT,
    EXTERNAL_SOURCE_CUBEMAP_UPLOAD_UNSUPPORTED_REASON, IBL_PMREM_RGBA16F_FORMAT,
    IBL_PMREM_RGBA16F_GPU_FORMAT, RGBA8_UNORM_FORMAT, RGBA8_UNORM_SRGB_FORMAT,
    ZCUBE_SOURCE_CUBEMAP_FORMAT, ZCUBE_SOURCE_CUBEMAP_GPU_FORMAT, ZCUBE_SOURCE_CUBEMAP_HEADER_SIZE,
};
pub use ui::{
    ui_asset_references, ui_v2_asset_references, UiAssetDocumentError, UiAssetDocumentResult,
    UiIconAsset, UiIconAssetDocumentError, UiIconAssetDocumentResult, UiIconSource,
    UiIconSourceKind, UiLayoutAsset, UiStyleAsset, UiThemeAsset, UiThemeAssetDocumentError,
    UiThemeAssetDocumentResult, UiV2AssetDocumentError, UiV2AssetDocumentResult,
    UiV2ComponentAsset, UiV2StyleAsset, UiV2ViewAsset, UiWidgetAsset,
};
