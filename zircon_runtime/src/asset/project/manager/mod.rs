use crate::core::resource::ResourceRegistry;

use crate::asset::registry::AssetRegistryIndex;
use crate::asset::{ArtifactStore, AssetImporter};

use super::{PackageAssetRegistry, ProjectManifest, ProjectPaths};

mod artifact_access;
mod asset_kind;
mod collect_files;
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
    importer: AssetImporter,
    artifact_store: ArtifactStore,
}
