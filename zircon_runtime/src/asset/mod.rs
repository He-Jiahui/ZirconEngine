//! Runtime asset subsystem: project manifests, loading, import, and pipeline runtime support.

mod module;

pub use module::{
    module_descriptor, AssetModule, ASSET_IO_DRIVER_NAME, ASSET_MANAGER_NAME, ASSET_MODULE_NAME,
    PROJECT_ASSET_MANAGER_NAME, RESOURCE_MANAGER_NAME,
};

pub mod artifact;
pub mod assets;
pub mod facade;
mod formats;
pub mod importer;
mod load;
pub mod pipeline;
pub mod project;
mod runtime_asset_path;
mod virtual_geometry_cook;
pub mod watch;

#[allow(unused_imports)]
pub(crate) use artifact::{ArtifactStore, LibraryCacheKey};
pub use assets::{
    asset_kind_for_imported_asset, AlphaMode, AnimationChannelAsset, AnimationChannelKeyAsset,
    AnimationChannelValueAsset, AnimationClipAsset, AnimationClipBoneTrackAsset,
    AnimationConditionOperatorAsset, AnimationEventTrackAsset, AnimationGraphAsset,
    AnimationGraphNodeAsset, AnimationGraphParameterAsset, AnimationInterpolationAsset,
    AnimationSequenceAsset, AnimationSequenceBindingAsset, AnimationSequenceTrackAsset,
    AnimationSkeletonAsset, AnimationSkeletonBoneAsset, AnimationStateAsset,
    AnimationStateMachineAsset, AnimationStateTransitionAsset, AnimationTransitionConditionAsset,
    DataAsset, DataAssetFormat, FontAsset, FontAssetError, ImportedAsset, MaterialAsset,
    MaterialGraphAsset, MaterialGraphLinkAsset, MaterialGraphNodeAsset, MaterialGraphNodeKindAsset,
    MaterialGraphParameterAsset, MaterialTextureSlotValue, ModelAsset, ModelPrimitiveAsset,
    NavMeshAreaCostAsset, NavMeshAsset, NavMeshGizmoTriangleAsset, NavMeshLinkAsset,
    NavMeshPolygonAsset, NavMeshTileAsset, NavigationSettingsAsset, PhysicsMaterialAsset,
    PrefabAsset, PrefabInstanceAsset, PrefabPropertyOverrideAsset, SceneAnimationGraphPlayerAsset,
    SceneAnimationPlayerAsset, SceneAnimationSequencePlayerAsset, SceneAnimationSkeletonAsset,
    SceneAnimationStateMachinePlayerAsset, SceneAsset, SceneCameraAsset, SceneCameraTargetAsset,
    SceneColliderAsset, SceneColliderShapeAsset, SceneDirectionalLightAsset, SceneEntityAsset,
    SceneJointAsset, SceneJointKindAsset, SceneMeshInstanceAsset, SceneMobilityAsset,
    ScenePointLightAsset, SceneRigidBodyAsset, SceneRigidBodyTypeAsset, SceneSpotLightAsset,
    SceneTerrainAsset, SceneTileMapAsset, SceneViewportRectAsset, ShaderAsset,
    ShaderDependencyAsset, ShaderEntryPointAsset, ShaderImportRedirectAsset,
    ShaderMaterialPropertyAsset, ShaderSourceFileAsset, ShaderSourceLanguage,
    ShaderTextureSlotAsset, SoundAsset, TerrainAsset, TerrainLayerAsset, TerrainLayerStackAsset,
    TextureArrayLayout, TextureAsset, TextureAssetDescriptor, TexturePayload, TileMapAsset,
    TileMapLayerAsset, TileMapProjectionAsset, TileSetAsset, TileSetTileAsset, TransformAsset,
    UiAssetDocumentError, UiLayoutAsset, UiStyleAsset, UiV2AssetDocumentError, UiV2ComponentAsset,
    UiV2StyleAsset, UiV2ViewAsset, UiWidgetAsset, VirtualGeometryAsset,
    VirtualGeometryClusterHeaderAsset, VirtualGeometryClusterPageHeaderAsset,
    VirtualGeometryDebugMetadataAsset, VirtualGeometryHierarchyNodeAsset,
    VirtualGeometryPageDependencyAsset, VirtualGeometryRootClusterRangeAsset, ZMaterialDocument,
    ZShaderDocument, ZShaderEntryPointDocument, ZShaderImportDocument, ZShaderTextureSlotDocument,
    RGBA8_UNORM_SRGB_FORMAT,
};
pub use facade::{
    Asset, AssetEvent, AssetEventReceiver, AssetLoadState, Assets, Handle,
    RecursiveDependencyLoadState,
};
pub use importer::{
    decode_texture_source_image, AssetImportContext, AssetImportError, AssetImportOutcome,
    AssetImporter, AssetImporterDescriptor, AssetImporterHandler, AssetImporterRegistry,
    AssetImporterRegistryError, AssetSchemaMigrationReport, AssetSchemaMigrator,
    DecodedTextureImage, DiagnosticOnlyAssetImporter, FunctionAssetImporter, ImportedAssetEntry,
    NativeAssetImportEntryMetadata, NativeAssetImportRequestMetadata,
    NativeAssetImportResponseMetadata, NativeAssetImporterHandler, StaticAssetSchemaMigrator,
};
pub use pipeline::manager::{
    resolve_asset_manager, AssetIoDriver, AssetManager, AssetManagerHandle, AssetPipelineInfo,
    AssetStatusRecord, ProjectAssetManager, ProjectInfo,
};
pub use pipeline::types::MeshVertex;
#[allow(unused_imports)]
pub(crate) use pipeline::types::{
    AssetRequest, CpuAssetPayload, CpuMeshPayload, CpuTexturePayload, MeshSource, TextureSource,
};
pub(crate) use pipeline::{types, worker_pool};
#[allow(unused_imports)]
pub(crate) use project::{
    AssetMetaDocument, AssetMetaEntry, AssetSourceUnit, PackageAssetRegistry, PreviewState,
    ProjectManager, ProjectManifest, ProjectPaths,
};
pub use runtime_asset_path::{
    runtime_asset_path, runtime_asset_path_with_dev_asset_root, runtime_asset_root,
};
pub use virtual_geometry_cook::{
    cook_virtual_geometry_from_mesh, encode_virtual_geometry_cook_binary_dump,
    format_virtual_geometry_cook_bvh_graph_dump, format_virtual_geometry_cook_inspection_dump,
    VirtualGeometryCookConfig,
};

pub type AssetId = crate::core::resource::ResourceId;
pub type AssetKind = crate::core::resource::ResourceKind;
pub type AssetReference = crate::core::resource::AssetReference;
pub type AssetUri = crate::core::resource::ResourceLocator;
pub type AssetUuid = crate::core::resource::AssetUuid;

#[cfg(test)]
mod tests;
