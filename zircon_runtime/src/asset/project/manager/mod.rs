use crate::core::resource::ResourceRegistry;
use crate::core::runtime::tasks::TaskPool;

use crate::asset::registry::AssetRegistryIndex;
use crate::asset::{ArtifactStore, AssetImporter};
use crate::scene::world::SceneComponentSerializerRegistry;
use scan_and_import::ShaderImportDependencyIndex;
use std::sync::{Arc, RwLock};

use super::{
    PackageAssetRegistry, ProjectCatalogInputGeneration, ProjectManifest, ProjectPaths,
    ProjectReferenceDiagnosticsStore,
};

#[path = "artifact_access.rs"]
mod artifact_access;
#[path = "asset_kind.rs"]
mod asset_kind;
#[path = "collect_files.rs"]
mod collect_files;
#[path = "deletion.rs"]
mod deletion;
#[path = "durable_transaction.rs"]
mod durable_transaction;
#[path = "editor_document_transaction.rs"]
mod editor_document_transaction;
#[path = "hash_bytes.rs"]
mod hash_bytes;
#[path = "importer_access.rs"]
mod importer_access;
#[path = "is_meta_sidecar.rs"]
mod is_meta_sidecar;
#[path = "load_or_create_meta.rs"]
mod load_or_create_meta;
#[path = "meta_path_for_source.rs"]
mod meta_path_for_source;
#[path = "open.rs"]
mod open;
#[path = "package_assets.rs"]
mod package_assets;
#[path = "persisted_reference.rs"]
mod persisted_reference;
#[path = "project_activation_identity.rs"]
mod project_activation_identity;
#[path = "registry_access.rs"]
mod registry_access;
#[path = "relocation.rs"]
mod relocation;
#[path = "scan_and_import.rs"]
mod scan_and_import;
#[path = "source_mtime_unix_ms.rs"]
mod source_mtime_unix_ms;
#[path = "source_path_for_uri.rs"]
mod source_path_for_uri;
#[path = "source_uri_for_path.rs"]
mod source_uri_for_path;

pub use editor_document_transaction::{
    EditorDocumentCommitFault, EditorDocumentCommitOutcome, EditorDocumentScope,
    ScopedRuntimeProjectManager,
};

pub(crate) use deletion::PreparedProjectSourceDeletion;
pub(crate) use load_or_create_meta::mint_meta_for_migration;
pub(crate) use relocation::PreparedProjectSourceRelocation;
pub(crate) use scan_and_import::{ImportSourceWatchEcho, PreparedWatchFileGeneration, WatchSourceCandidate};

pub use project_activation_identity::ProjectActivationIdentity;

#[derive(Clone, Debug)]
pub struct ProjectManager {
    activation_identity: ProjectActivationIdentity,
    paths: ProjectPaths,
    manifest: ProjectManifest,
    registry: ResourceRegistry,
    asset_registry: Arc<AssetRegistryIndex>,
    package_assets: PackageAssetRegistry,
    catalog_input_generation: Arc<ProjectCatalogInputGeneration>,
    reference_diagnostics: Arc<ProjectReferenceDiagnosticsStore>,
    importer: AssetImporter,
    artifact_store: ArtifactStore,
    shader_import_dependencies: ShaderImportDependencyIndex,
    environment_ibl_parallel_executor: Option<TaskPool>,
    /// Provider codecs owned by this project/runtime lifecycle.
    ///
    /// Clones of `ProjectManager` share this state, while separate projects receive
    /// independent registries. Scene persistence takes a snapshot from this owner
    /// for each operation, so it never falls back to a fresh builtin registry.
    scene_component_registry: Arc<RwLock<SceneComponentSerializerRegistry>>,
}
