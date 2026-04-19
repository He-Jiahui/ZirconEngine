//! Minimal asynchronous asset pipeline for textures and simple meshes.

pub mod artifact;
pub mod assets;
pub mod editor;
mod formats;
pub mod importer;
mod load;
pub mod pipeline;
pub mod project;
pub mod watch;

#[allow(unused_imports)]
pub(crate) use artifact::{ArtifactStore, LibraryCacheKey};
#[allow(unused_imports)]
pub(crate) use assets::{
    AlphaMode, ImportedAsset, MaterialAsset, ModelAsset, ModelPrimitiveAsset, SceneAsset,
    SceneCameraAsset, SceneDirectionalLightAsset, SceneEntityAsset, SceneMeshInstanceAsset,
    SceneMobilityAsset, ShaderAsset, TextureAsset, TransformAsset, UiAssetDocumentError,
    UiLayoutAsset, UiStyleAsset, UiWidgetAsset,
};
#[allow(unused_imports)]
pub(crate) use editor::{
    AssetCatalogRecord, DefaultEditorAssetManager, EditorAssetCatalogRecord,
    EditorAssetCatalogSnapshotRecord, EditorAssetChange, EditorAssetChangeKind,
    EditorAssetChangeRecord, EditorAssetDetailsRecord, EditorAssetFolderRecord,
    EditorAssetManager, EditorAssetManagerHandle, EditorAssetReferenceRecord,
    PreviewArtifactKey, PreviewCache, PreviewScheduler, ReferenceGraph,
    resolve_editor_asset_manager,
};
#[allow(unused_imports)]
pub(crate) use importer::{AssetImportError, AssetImporter};
pub(crate) use pipeline::{types, worker_pool};
#[allow(unused_imports)]
pub(crate) use pipeline::manager::{
    ASSET_MANAGER_NAME, EDITOR_ASSET_MANAGER_NAME, PROJECT_ASSET_MANAGER_NAME, AssetIoDriver,
    AssetManager, AssetManagerHandle, AssetPipelineInfo, AssetStatusRecord, ProjectAssetManager,
    ProjectInfo, resolve_asset_manager,
};
#[allow(unused_imports)]
pub(crate) use pipeline::types::{
    AssetRequest, CpuAssetPayload, CpuMeshPayload, CpuTexturePayload, MeshSource, MeshVertex,
    TextureSource,
};
#[allow(unused_imports)]
pub(crate) use project::{
    AssetMetaDocument, PreviewState, ProjectManager, ProjectManifest, ProjectPaths,
};

pub type AssetId = zircon_resource::ResourceId;
pub type AssetKind = zircon_resource::ResourceKind;
pub type AssetReference = zircon_resource::AssetReference;
pub type AssetUri = zircon_resource::ResourceLocator;
pub type AssetUuid = zircon_resource::AssetUuid;

#[cfg(test)]
mod tests;
