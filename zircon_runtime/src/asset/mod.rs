//! Runtime asset subsystem: project manifests, loading, import, and pipeline runtime support.

mod module;

pub use module::{
    module_descriptor, AssetModule, ASSET_IO_DRIVER_NAME, ASSET_MANAGER_NAME, ASSET_MODULE_NAME,
    PROJECT_ASSET_MANAGER_NAME, RESOURCE_MANAGER_NAME,
};

pub mod artifact;
pub mod assets;
mod formats;
pub mod importer;
mod load;
pub mod pipeline;
pub mod project;
pub mod watch;

#[allow(unused_imports)]
pub(crate) use artifact::{ArtifactStore, LibraryCacheKey};
pub use assets::{
    AlphaMode, AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationClipAsset, AnimationClipBoneTrackAsset, AnimationConditionOperatorAsset,
    AnimationGraphAsset, AnimationGraphNodeAsset, AnimationGraphParameterAsset,
    AnimationInterpolationAsset, AnimationSequenceAsset, AnimationSequenceBindingAsset,
    AnimationSequenceTrackAsset, AnimationSkeletonAsset, AnimationSkeletonBoneAsset,
    AnimationStateAsset, AnimationStateMachineAsset, AnimationStateTransitionAsset,
    AnimationTransitionConditionAsset, FontAsset, FontAssetError, ImportedAsset, MaterialAsset,
    ModelAsset, ModelPrimitiveAsset, PhysicsMaterialAsset, SceneAnimationGraphPlayerAsset,
    SceneAnimationPlayerAsset, SceneAnimationSequencePlayerAsset, SceneAnimationSkeletonAsset,
    SceneAnimationStateMachinePlayerAsset, SceneAsset, SceneCameraAsset, SceneColliderAsset,
    SceneColliderShapeAsset, SceneDirectionalLightAsset, SceneEntityAsset, SceneJointAsset,
    SceneJointKindAsset, SceneMeshInstanceAsset, SceneMobilityAsset, ScenePointLightAsset,
    SceneRigidBodyAsset, SceneRigidBodyTypeAsset, SceneSpotLightAsset, ShaderAsset, SoundAsset,
    TextureAsset, TransformAsset, UiAssetDocumentError, UiLayoutAsset, UiStyleAsset, UiWidgetAsset,
    VirtualGeometryAsset, VirtualGeometryClusterHeaderAsset, VirtualGeometryClusterPageHeaderAsset,
    VirtualGeometryDebugMetadataAsset, VirtualGeometryHierarchyNodeAsset,
    VirtualGeometryRootClusterRangeAsset,
};
pub use importer::{AssetImportError, AssetImporter};
pub use pipeline::manager::{
    resolve_asset_manager, AssetIoDriver, AssetManager, AssetManagerHandle, AssetPipelineInfo,
    AssetStatusRecord, ProjectAssetManager, ProjectInfo,
};
#[allow(unused_imports)]
pub(crate) use pipeline::types::{
    AssetRequest, CpuAssetPayload, CpuMeshPayload, CpuTexturePayload, MeshSource, MeshVertex,
    TextureSource,
};
pub(crate) use pipeline::{types, worker_pool};
#[allow(unused_imports)]
pub(crate) use project::{
    AssetMetaDocument, PreviewState, ProjectManager, ProjectManifest, ProjectPaths,
};

pub type AssetId = crate::core::resource::ResourceId;
pub type AssetKind = crate::core::resource::ResourceKind;
pub type AssetReference = crate::core::resource::AssetReference;
pub type AssetUri = crate::core::resource::ResourceLocator;
pub type AssetUuid = crate::core::resource::AssetUuid;

#[cfg(test)]
mod tests;
