use std::collections::HashMap;
use std::fs;

use zircon_resource::{ResourceRecord, ResourceRegistry};

use crate::project::PreviewState;
use crate::{AssetId, AssetImportError};

use super::{
    asset_kind::asset_kind, collect_files::collect_files, hash_bytes::hash_bytes,
    load_or_create_meta::load_or_create_meta, meta_path_for_source::meta_path_for_source,
    source_mtime_unix_ms::source_mtime_unix_ms, ProjectManager,
};

const IMPORTER_VERSION: u32 = 1;

impl ProjectManager {
    pub fn scan_and_import(&mut self) -> Result<Vec<ResourceRecord>, AssetImportError> {
        let mut files = Vec::new();
        collect_files(self.paths.assets_root(), &mut files)?;
        files.sort();

        let mut registry = ResourceRegistry::default();
        let mut asset_ids_by_uuid = HashMap::new();
        let mut asset_uuids_by_id = HashMap::new();
        let mut imported = Vec::with_capacity(files.len());

        for file in files {
            let uri = self.source_uri_for_path(&file)?;
            let imported_asset = self.importer.import_from_source(&file, &uri)?;
            let kind = asset_kind(&imported_asset);
            let source_bytes = fs::read(&file)?;
            let source_hash = hash_bytes(&source_bytes);
            let source_mtime_unix_ms = source_mtime_unix_ms(&file)?;
            let meta_path = meta_path_for_source(&file);
            let meta_exists = meta_path.exists();
            let mut meta = load_or_create_meta(&meta_path, &uri, kind)?;
            let previous_meta = meta.clone();
            let previous_source_hash = meta.source_hash.clone();
            meta.primary_locator = uri.clone();
            meta.kind = kind;
            meta.source_hash = source_hash.clone();
            meta.source_mtime_unix_ms = source_mtime_unix_ms;
            if meta.source_hash != previous_source_hash {
                meta.preview_state = PreviewState::Dirty;
            }
            if !meta_exists || meta != previous_meta {
                meta.save(&meta_path)?;
            }

            let asset_id = AssetId::from_asset_uuid_label(meta.asset_uuid, uri.label());
            let artifact_uri = self.artifact_store.write(
                &self.paths,
                &ResourceRecord::new(asset_id, kind, uri.clone()),
                &imported_asset,
            )?;
            let metadata = ResourceRecord::new(asset_id, kind, uri)
                .with_source_hash(source_hash)
                .with_importer_version(IMPORTER_VERSION)
                .with_config_hash("")
                .with_artifact_locator(artifact_uri);
            registry.upsert(metadata.clone());
            asset_ids_by_uuid.insert(meta.asset_uuid, asset_id);
            asset_uuids_by_id.insert(asset_id, meta.asset_uuid);
            imported.push(metadata);
        }

        self.registry = registry;
        self.asset_ids_by_uuid = asset_ids_by_uuid;
        self.asset_uuids_by_id = asset_uuids_by_id;
        Ok(imported)
    }
}
