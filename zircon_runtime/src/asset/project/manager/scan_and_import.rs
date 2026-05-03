use std::collections::HashMap;
use std::fs;

use crate::core::resource::{ResourceDiagnostic, ResourceRecord, ResourceRegistry, ResourceState};

use crate::asset::project::PreviewState;
use crate::asset::{
    AssetId, AssetImportError, AssetImportOutcome, AssetImporterDescriptor, AssetKind,
};

use super::{
    asset_kind::asset_kind, collect_files::collect_files, hash_bytes::hash_bytes,
    load_or_create_meta::load_or_create_meta, meta_path_for_source::meta_path_for_source,
    source_mtime_unix_ms::source_mtime_unix_ms, ProjectManager,
};

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
            let source_bytes = fs::read(&file)?;
            let source_hash = hash_bytes(&source_bytes);
            let source_mtime_unix_ms = source_mtime_unix_ms(&file)?;
            let descriptor = self.importer.descriptor_for_source(&file).ok();
            let fallback_kind = descriptor
                .as_ref()
                .map(|descriptor| descriptor.output_kind)
                .unwrap_or(AssetKind::Data);
            let meta_path = meta_path_for_source(&file);
            let meta_exists = meta_path.exists();
            let mut meta = load_or_create_meta(&meta_path, &uri, fallback_kind)?;
            let previous_meta = meta.clone();
            let import_settings = meta.import_settings.clone();
            let config_hash = config_hash_for_settings(&import_settings);
            let asset_id = AssetId::from_asset_uuid_label(meta.asset_uuid, uri.label());

            if let Some(metadata) = self.restore_imported_artifact(
                &uri,
                &mut meta,
                meta_exists,
                &previous_meta,
                source_hash.clone(),
                source_mtime_unix_ms,
                config_hash.clone(),
                descriptor.as_ref(),
                fallback_kind,
                asset_id,
            )? {
                registry.upsert(metadata.clone());
                asset_ids_by_uuid.insert(meta.asset_uuid, asset_id);
                asset_uuids_by_id.insert(asset_id, meta.asset_uuid);
                imported.push(metadata);
                continue;
            }

            let import_result =
                self.importer
                    .import_bytes(&file, &uri, source_bytes, import_settings);
            let metadata = match import_result {
                Ok(outcome) => self.finish_successful_import(
                    &uri,
                    &mut meta,
                    meta_exists,
                    &previous_meta,
                    source_hash.clone(),
                    source_mtime_unix_ms,
                    config_hash,
                    descriptor.as_ref(),
                    asset_id,
                    outcome,
                )?,
                Err(error) => self.finish_failed_import(
                    &uri,
                    &mut meta,
                    meta_exists,
                    &previous_meta,
                    source_hash.clone(),
                    source_mtime_unix_ms,
                    config_hash,
                    descriptor.as_ref(),
                    fallback_kind,
                    asset_id,
                    error,
                )?,
            };
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

    #[allow(clippy::too_many_arguments)]
    fn restore_imported_artifact(
        &self,
        uri: &crate::asset::AssetUri,
        meta: &mut crate::asset::project::AssetMetaDocument,
        meta_exists: bool,
        previous_meta: &crate::asset::project::AssetMetaDocument,
        source_hash: String,
        source_mtime_unix_ms: u64,
        config_hash: String,
        descriptor: Option<&AssetImporterDescriptor>,
        fallback_kind: AssetKind,
        asset_id: AssetId,
    ) -> Result<Option<ResourceRecord>, AssetImportError> {
        if meta.preview_state != PreviewState::Ready
            || meta.source_hash != source_hash
            || meta.config_hash != config_hash
            || !importer_contract_matches(meta, descriptor)
        {
            return Ok(None);
        }

        let Some(artifact_uri) = meta.artifact_locator.clone() else {
            return Ok(None);
        };
        if self
            .artifact_store
            .read(&self.paths, &artifact_uri)
            .is_err()
        {
            return Ok(None);
        }

        meta.primary_locator = uri.clone();
        if meta.kind == AssetKind::Data && descriptor.is_some() {
            meta.kind = fallback_kind;
        }
        meta.source_mtime_unix_ms = source_mtime_unix_ms;
        if !meta_exists || meta != previous_meta {
            meta.save(meta_path_for_source(&self.source_path_for_uri(uri)?))?;
        }

        Ok(Some(
            ResourceRecord::new(asset_id, meta.kind, uri.clone())
                .with_source_hash(source_hash)
                .with_importer_id(meta.importer_id.clone())
                .with_importer_version(meta.importer_version)
                .with_config_hash(config_hash)
                .with_artifact_locator(artifact_uri)
                .with_state(ResourceState::Ready),
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn finish_successful_import(
        &self,
        uri: &crate::asset::AssetUri,
        meta: &mut crate::asset::project::AssetMetaDocument,
        meta_exists: bool,
        previous_meta: &crate::asset::project::AssetMetaDocument,
        source_hash: String,
        source_mtime_unix_ms: u64,
        config_hash: String,
        descriptor: Option<&AssetImporterDescriptor>,
        asset_id: AssetId,
        outcome: AssetImportOutcome,
    ) -> Result<ResourceRecord, AssetImportError> {
        let kind = asset_kind(&outcome.imported_asset);
        apply_importer_metadata(meta, descriptor);
        if let Some(migration) = &outcome.migration_report {
            meta.source_schema_version = migration.source_schema_version;
            meta.target_schema_version = Some(migration.target_schema_version);
            meta.migration_summary = migration.summary.clone();
        } else {
            clear_schema_migration_metadata(meta);
        }
        meta.primary_locator = uri.clone();
        meta.kind = kind;
        meta.config_hash = config_hash.clone();
        meta.source_hash = source_hash.clone();
        meta.source_mtime_unix_ms = source_mtime_unix_ms;
        meta.preview_state = PreviewState::Ready;

        let artifact_uri = self.artifact_store.write(
            &self.paths,
            &ResourceRecord::new(asset_id, kind, uri.clone()),
            &outcome.imported_asset,
        )?;
        meta.artifact_locator = Some(artifact_uri.clone());
        if !meta_exists || meta != previous_meta {
            meta.save(meta_path_for_source(&self.source_path_for_uri(uri)?))?;
        }

        Ok(ResourceRecord::new(asset_id, kind, uri.clone())
            .with_source_hash(source_hash)
            .with_importer_id(meta.importer_id.clone())
            .with_importer_version(meta.importer_version)
            .with_config_hash(config_hash)
            .with_artifact_locator(artifact_uri)
            .with_state(ResourceState::Ready)
            .with_diagnostics(outcome.diagnostics))
    }

    #[allow(clippy::too_many_arguments)]
    fn finish_failed_import(
        &self,
        uri: &crate::asset::AssetUri,
        meta: &mut crate::asset::project::AssetMetaDocument,
        meta_exists: bool,
        previous_meta: &crate::asset::project::AssetMetaDocument,
        source_hash: String,
        source_mtime_unix_ms: u64,
        config_hash: String,
        descriptor: Option<&AssetImporterDescriptor>,
        kind: AssetKind,
        asset_id: AssetId,
        error: AssetImportError,
    ) -> Result<ResourceRecord, AssetImportError> {
        apply_importer_metadata(meta, descriptor);
        clear_schema_migration_metadata(meta);
        meta.primary_locator = uri.clone();
        meta.kind = kind;
        meta.artifact_locator = None;
        meta.config_hash = config_hash.clone();
        meta.source_hash = source_hash.clone();
        meta.source_mtime_unix_ms = source_mtime_unix_ms;
        meta.preview_state = PreviewState::Error;
        if !meta_exists || meta != previous_meta {
            meta.save(meta_path_for_source(&self.source_path_for_uri(uri)?))?;
        }

        Ok(ResourceRecord::new(asset_id, kind, uri.clone())
            .with_source_hash(source_hash)
            .with_importer_id(meta.importer_id.clone())
            .with_importer_version(meta.importer_version)
            .with_config_hash(config_hash)
            .with_state(ResourceState::Error)
            .with_diagnostics(vec![ResourceDiagnostic::error(error.to_string())]))
    }
}

fn clear_schema_migration_metadata(meta: &mut crate::asset::project::AssetMetaDocument) {
    meta.source_schema_version = None;
    meta.target_schema_version = None;
    meta.migration_summary.clear();
}

fn apply_importer_metadata(
    meta: &mut crate::asset::project::AssetMetaDocument,
    descriptor: Option<&AssetImporterDescriptor>,
) {
    if let Some(descriptor) = descriptor {
        meta.importer_id = descriptor.id.clone();
        meta.importer_version = descriptor.importer_version;
    } else {
        meta.importer_id.clear();
        meta.importer_version = 0;
    }
}

fn importer_contract_matches(
    meta: &crate::asset::project::AssetMetaDocument,
    descriptor: Option<&AssetImporterDescriptor>,
) -> bool {
    descriptor
        .map(|descriptor| {
            meta.importer_id == descriptor.id
                && meta.importer_version == descriptor.importer_version
        })
        .unwrap_or_else(|| !meta.importer_id.is_empty())
}

fn config_hash_for_settings(settings: &toml::Table) -> String {
    toml::to_string(settings)
        .map(|document| hash_bytes(document.as_bytes()))
        .unwrap_or_default()
}
