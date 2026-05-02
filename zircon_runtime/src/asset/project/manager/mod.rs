use std::collections::HashMap;

use crate::core::resource::ResourceRegistry;

use crate::asset::{ArtifactStore, AssetId, AssetImporter, AssetUuid};

use super::{ProjectManifest, ProjectPaths};

mod artifact_access;
mod asset_kind;
mod asset_lookup;
mod collect_files;
mod hash_bytes;
mod importer_access;
mod is_meta_sidecar;
mod load_or_create_meta;
mod meta_path_for_source;
mod open;
mod registry_access;
mod scan_and_import;
mod source_mtime_unix_ms;
mod source_path_for_uri;
mod source_uri_for_path;

#[derive(Clone, Debug)]
pub struct ProjectManager {
    paths: ProjectPaths,
    manifest: ProjectManifest,
    registry: ResourceRegistry,
    asset_ids_by_uuid: HashMap<AssetUuid, AssetId>,
    asset_uuids_by_id: HashMap<AssetId, AssetUuid>,
    importer: AssetImporter,
    artifact_store: ArtifactStore,
}
