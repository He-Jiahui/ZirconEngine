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

pub(crate) use pipeline::{types, worker_pool};

pub use artifact::{ArtifactStore, LibraryCacheKey};
pub use assets::{
    AlphaMode, ImportedAsset, MaterialAsset, ModelAsset, ModelPrimitiveAsset, SceneAsset,
    SceneCameraAsset, SceneDirectionalLightAsset, SceneEntityAsset, SceneMeshInstanceAsset,
    SceneMobilityAsset, ShaderAsset, TextureAsset, TransformAsset, UiAssetDocumentError,
    UiLayoutAsset, UiStyleAsset, UiWidgetAsset,
};
pub use editor::{
    AssetCatalogRecord, DefaultEditorAssetManager, PreviewArtifactKey, PreviewCache,
    PreviewScheduler, ReferenceGraph,
};
pub use importer::{AssetImportError, AssetImporter};
pub use pipeline::manager::{
    module_descriptor, AssetIoDriver, ProjectAssetManager, ASSET_IO_DRIVER_NAME,
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
pub use zircon_resource::{
    AssetReference, AssetUuid, MaterialMarker, ModelMarker, ResourceData, ResourceDiagnostic,
    ResourceDiagnosticSeverity, ResourceEvent, ResourceEventKind, ResourceHandle, ResourceId,
    ResourceInspectorAdapterKey, ResourceIo, ResourceIoError, ResourceKind, ResourceLease,
    ResourceLocator, ResourceLocatorError, ResourceManager, ResourceMarker, ResourceRecord,
    ResourceRegistry, ResourceRuntimeInfo, ResourceScheme, ResourceState, ResourceTypeDescriptor,
    RuntimeResourceState, SceneMarker, ShaderMarker, TextureMarker, UiLayoutMarker, UiStyleMarker,
    UiWidgetMarker, UntypedResourceHandle,
};

pub type AssetId = ResourceId;
pub type AssetKind = ResourceKind;
pub type AssetMetadata = ResourceRecord;
pub type AssetRegistry = ResourceRegistry;
pub type AssetUri = ResourceLocator;
pub type AssetUriError = ResourceLocatorError;
pub type AssetUriScheme = ResourceScheme;

#[cfg(test)]
mod tests;
