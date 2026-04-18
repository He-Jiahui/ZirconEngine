//! Minimal asynchronous asset pipeline for textures and simple meshes.

mod artifact;
mod assets;
mod editor;
mod formats;
mod importer;
mod load;
mod pipeline;
mod project;
mod watch;

use zircon_module::{EngineModule, ModuleDescriptor};

pub(crate) use pipeline::{types, worker_pool};

pub use artifact::{ArtifactStore, LibraryCacheKey};
pub use assets::{
    AlphaMode, ImportedAsset, MaterialAsset, ModelAsset, ModelPrimitiveAsset, SceneAsset,
    SceneCameraAsset, SceneDirectionalLightAsset, SceneEntityAsset, SceneMeshInstanceAsset,
    SceneMobilityAsset, ShaderAsset, TextureAsset, TransformAsset, UiAssetDocumentError,
    UiLayoutAsset, UiStyleAsset, UiWidgetAsset,
};
pub use editor::{
    resolve_editor_asset_manager, AssetCatalogRecord, DefaultEditorAssetManager,
    EditorAssetCatalogRecord, EditorAssetCatalogSnapshotRecord, EditorAssetChange,
    EditorAssetChangeKind, EditorAssetChangeRecord, EditorAssetDetailsRecord,
    EditorAssetFolderRecord, EditorAssetManager, EditorAssetManagerHandle,
    EditorAssetReferenceRecord, PreviewArtifactKey, PreviewCache, PreviewScheduler, ReferenceGraph,
};
pub use importer::{AssetImportError, AssetImporter};
pub use pipeline::manager::{
    module_descriptor, resolve_asset_manager, AssetIoDriver, AssetManager, AssetManagerHandle,
    AssetPipelineInfo, AssetStatusRecord, ProjectAssetManager, ProjectInfo, ASSET_IO_DRIVER_NAME,
    ASSET_MANAGER_NAME, ASSET_MODULE_NAME, EDITOR_ASSET_MANAGER_NAME, PROJECT_ASSET_MANAGER_NAME,
    RESOURCE_MANAGER_NAME,
};
pub use pipeline::types::{
    AssetRequest, CpuAssetPayload, CpuMeshPayload, CpuTexturePayload, MeshSource, MeshVertex,
    TextureSource,
};
pub use pipeline::worker_pool::AssetWorkerPool;
pub use project::{AssetMetaDocument, PreviewState};
pub use project::{ProjectManager, ProjectManifest, ProjectPaths};
pub use watch::{AssetChange, AssetChangeKind, AssetWatchEvent, AssetWatcher};

pub type AssetId = zircon_resource::ResourceId;
pub type AssetKind = zircon_resource::ResourceKind;
pub type AssetMetadata = zircon_resource::ResourceRecord;
pub type AssetReference = zircon_resource::AssetReference;
pub type AssetRegistry = zircon_resource::ResourceRegistry;
pub type AssetUri = zircon_resource::ResourceLocator;
pub type AssetUriError = zircon_resource::ResourceLocatorError;
pub type AssetUriScheme = zircon_resource::ResourceScheme;
pub type AssetUuid = zircon_resource::AssetUuid;

#[derive(Clone, Copy, Debug, Default)]
pub struct AssetModule;

impl EngineModule for AssetModule {
    fn module_name(&self) -> &'static str {
        ASSET_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Project asset pipeline, import workers, and resource indexing"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}

#[cfg(test)]
mod tests;
