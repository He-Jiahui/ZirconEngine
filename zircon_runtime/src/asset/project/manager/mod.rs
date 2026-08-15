use crate::core::resource::ResourceRegistry;
use crate::core::runtime::tasks::TaskPool;

use crate::asset::registry::AssetRegistryIndex;
use crate::asset::{ArtifactStore, AssetImporter};
use scan_and_import::ShaderImportDependencyIndex;
use std::sync::Arc;

use super::{PackageAssetRegistry, ProjectCatalogInputGeneration, ProjectManifest, ProjectPaths};

mod artifact_access;
mod asset_kind;
mod collect_files;
mod durable_transaction;
mod hash_bytes;
mod importer_access;
mod is_meta_sidecar;
mod load_or_create_meta;
mod meta_path_for_source;
mod open;
mod package_assets;
mod persisted_reference;
mod registry_access;
mod scan_and_import;
mod source_mtime_unix_ms;
mod source_path_for_uri;
mod source_uri_for_path;

pub(crate) use load_or_create_meta::mint_meta_for_migration;

#[derive(Clone, Debug)]
pub struct ProjectManager {
    paths: ProjectPaths,
    manifest: ProjectManifest,
    registry: ResourceRegistry,
    asset_registry: AssetRegistryIndex,
    package_assets: PackageAssetRegistry,
    catalog_input_generation: Arc<ProjectCatalogInputGeneration>,
    importer: AssetImporter,
    artifact_store: ArtifactStore,
    shader_import_dependencies: ShaderImportDependencyIndex,
    environment_ibl_parallel_executor: Option<TaskPool>,
}
