use std::collections::{HashMap, HashSet};
use std::fs;

use crate::core::resource::{ResourceDiagnostic, ResourceRecord, ResourceRegistry, ResourceState};

use crate::asset::project::{AssetMetaEntry, PreviewState};
use crate::asset::{
    AssetId, AssetImportError, AssetImportOutcome, AssetImporterDescriptor, AssetKind,
    ImportedAssetEntry,
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
        let mut dependencies_by_id = HashMap::new();
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
            let root_asset_id = AssetId::from_asset_uuid_label(meta.asset_uuid, None);

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
            )? {
                for record in metadata {
                    let asset_id = record.id();
                    dependencies_by_id.insert(
                        asset_id,
                        dependencies_for_entry(&meta, record.primary_locator()),
                    );
                    registry.upsert(record.clone());
                    if record.primary_locator().label().is_none() {
                        asset_ids_by_uuid.insert(meta.asset_uuid, asset_id);
                    }
                    asset_uuids_by_id.insert(asset_id, meta.asset_uuid);
                    imported.push(record);
                }
                continue;
            }

            let import_result =
                self.importer
                    .import_bytes(&file, &uri, source_bytes, import_settings);
            let metadata = match import_result {
                Ok(outcome) => match validate_import_entries(&uri, &outcome) {
                    Ok(()) => self.finish_successful_import(
                        &uri,
                        &mut meta,
                        meta_exists,
                        &previous_meta,
                        source_hash.clone(),
                        source_mtime_unix_ms,
                        config_hash,
                        descriptor.as_ref(),
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
                        root_asset_id,
                        error,
                    )?,
                },
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
                    root_asset_id,
                    error,
                )?,
            };
            for record in metadata {
                let asset_id = record.id();
                dependencies_by_id.insert(
                    asset_id,
                    dependencies_for_entry(&meta, record.primary_locator()),
                );
                registry.upsert(record.clone());
                if record.primary_locator().label().is_none() {
                    asset_ids_by_uuid.insert(meta.asset_uuid, asset_id);
                }
                asset_uuids_by_id.insert(asset_id, meta.asset_uuid);
                imported.push(record);
            }
        }

        resolve_imported_dependencies(&mut registry, &mut imported, &dependencies_by_id);

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
    ) -> Result<Option<Vec<ResourceRecord>>, AssetImportError> {
        if meta.preview_state != PreviewState::Ready
            || meta.source_hash != source_hash
            || meta.config_hash != config_hash
            || !importer_contract_matches(meta, descriptor)
        {
            return Ok(None);
        }

        if meta.entries.is_empty() {
            let Some(artifact_uri) = meta.artifact_locator.clone() else {
                return Ok(None);
            };
            meta.entries = vec![AssetMetaEntry {
                locator: uri.clone(),
                kind: meta.kind,
                artifact_locator: Some(artifact_uri),
                dependencies: meta.dependencies.clone(),
            }];
        }

        for entry in &meta.entries {
            let Some(artifact_uri) = &entry.artifact_locator else {
                return Ok(None);
            };
            if self.artifact_store.read(&self.paths, artifact_uri).is_err() {
                return Ok(None);
            }
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
            meta.entries
                .iter()
                .map(|entry| {
                    let entry_asset_id = asset_id_for_meta_entry(meta.asset_uuid, &entry.locator);
                    let mut record =
                        ResourceRecord::new(entry_asset_id, entry.kind, entry.locator.clone())
                            .with_source_hash(source_hash.clone())
                            .with_importer_id(meta.importer_id.clone())
                            .with_importer_version(meta.importer_version)
                            .with_config_hash(config_hash.clone())
                            .with_state(ResourceState::Ready);
                    if let Some(artifact_uri) = entry.artifact_locator.clone() {
                        record = record.with_artifact_locator(artifact_uri);
                    }
                    record
                })
                .collect(),
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
        outcome: AssetImportOutcome,
    ) -> Result<Vec<ResourceRecord>, AssetImportError> {
        let root_entry = outcome.root_entry().ok_or_else(|| {
            AssetImportError::Parse(format!("importer did not return a root entry for {uri}"))
        })?;
        let kind = asset_kind(&root_entry.asset);
        apply_importer_metadata(meta, descriptor);
        if let Some(migration) = &root_entry.migration_report {
            meta.source_schema_version = migration.source_schema_version;
            meta.target_schema_version = Some(migration.target_schema_version);
            meta.migration_summary = migration.summary.clone();
        } else {
            clear_schema_migration_metadata(meta);
        }
        meta.primary_locator = uri.clone();
        meta.kind = kind;
        meta.artifact_locator = None;
        meta.dependencies = root_entry.dependencies.clone();
        meta.config_hash = config_hash.clone();
        meta.source_hash = source_hash.clone();
        meta.source_mtime_unix_ms = source_mtime_unix_ms;
        meta.preview_state = PreviewState::Ready;

        let mut entries = Vec::with_capacity(outcome.entries.len());
        let mut records = Vec::with_capacity(outcome.entries.len());
        for entry in outcome.entries {
            let entry_kind = asset_kind(&entry.asset);
            let entry_asset_id = asset_id_for_import_entry(meta.asset_uuid, &entry);
            let artifact_record =
                ResourceRecord::new(entry_asset_id, entry_kind, entry.locator.clone());
            let artifact_uri =
                self.artifact_store
                    .write(&self.paths, &artifact_record, &entry.asset)?;
            if entry.locator.label().is_none() {
                meta.artifact_locator = Some(artifact_uri.clone());
            }
            entries.push(AssetMetaEntry {
                locator: entry.locator.clone(),
                kind: entry_kind,
                artifact_locator: Some(artifact_uri.clone()),
                dependencies: entry.dependencies.clone(),
            });
            records.push(
                ResourceRecord::new(entry_asset_id, entry_kind, entry.locator)
                    .with_source_hash(source_hash.clone())
                    .with_importer_id(meta.importer_id.clone())
                    .with_importer_version(meta.importer_version)
                    .with_config_hash(config_hash.clone())
                    .with_artifact_locator(artifact_uri)
                    .with_state(ResourceState::Ready)
                    .with_diagnostics(entry.diagnostics),
            );
        }
        meta.entries = entries;
        if !meta_exists || meta != previous_meta {
            meta.save(meta_path_for_source(&self.source_path_for_uri(uri)?))?;
        }

        Ok(records)
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
    ) -> Result<Vec<ResourceRecord>, AssetImportError> {
        apply_importer_metadata(meta, descriptor);
        clear_schema_migration_metadata(meta);
        meta.primary_locator = uri.clone();
        meta.kind = kind;
        meta.artifact_locator = None;
        meta.dependencies.clear();
        meta.entries.clear();
        meta.config_hash = config_hash.clone();
        meta.source_hash = source_hash.clone();
        meta.source_mtime_unix_ms = source_mtime_unix_ms;
        meta.preview_state = PreviewState::Error;
        if !meta_exists || meta != previous_meta {
            meta.save(meta_path_for_source(&self.source_path_for_uri(uri)?))?;
        }

        Ok(vec![ResourceRecord::new(asset_id, kind, uri.clone())
            .with_source_hash(source_hash)
            .with_importer_id(meta.importer_id.clone())
            .with_importer_version(meta.importer_version)
            .with_config_hash(config_hash)
            .with_state(ResourceState::Error)
            .with_diagnostics(vec![ResourceDiagnostic::error(
                error.to_string(),
            )])])
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

#[derive(Default)]
struct ResolvedDependencies {
    dependency_ids: Vec<AssetId>,
    diagnostics: Vec<ResourceDiagnostic>,
}

fn resolve_dependencies(
    dependencies: &[crate::asset::AssetUri],
    registry: &ResourceRegistry,
) -> ResolvedDependencies {
    let mut resolved = ResolvedDependencies::default();
    for dependency in dependencies {
        if let Some(record) = registry.get_by_locator(dependency) {
            if !resolved.dependency_ids.contains(&record.id()) {
                resolved.dependency_ids.push(record.id());
            }
        } else {
            resolved.diagnostics.push(ResourceDiagnostic::error(format!(
                "unresolved asset dependency {dependency}"
            )));
        }
    }
    resolved
}

fn resolve_imported_dependencies(
    registry: &mut ResourceRegistry,
    imported: &mut [ResourceRecord],
    dependencies_by_id: &HashMap<AssetId, Vec<crate::asset::AssetUri>>,
) {
    let resolved_by_id = dependencies_by_id
        .iter()
        .map(|(id, dependencies)| (*id, resolve_dependencies(dependencies, registry)))
        .collect::<HashMap<_, _>>();

    for record in imported.iter_mut() {
        apply_resolved_dependencies(record, &resolved_by_id);
        registry.upsert(record.clone());
    }
}

fn apply_resolved_dependencies(
    record: &mut ResourceRecord,
    resolved_by_id: &HashMap<AssetId, ResolvedDependencies>,
) {
    let Some(resolved) = resolved_by_id.get(&record.id()) else {
        return;
    };
    record.dependency_ids = resolved.dependency_ids.clone();
    record
        .diagnostics
        .extend(resolved.diagnostics.iter().cloned());
}

fn dependencies_for_entry(
    meta: &crate::asset::project::AssetMetaDocument,
    locator: &crate::asset::AssetUri,
) -> Vec<crate::asset::AssetUri> {
    meta.entries
        .iter()
        .find(|entry| &entry.locator == locator)
        .map(|entry| entry.dependencies.clone())
        .unwrap_or_else(|| meta.dependencies.clone())
}

fn asset_id_for_meta_entry(
    uuid: crate::asset::AssetUuid,
    locator: &crate::asset::AssetUri,
) -> AssetId {
    AssetId::from_asset_uuid_label(uuid, locator.label())
}

fn asset_id_for_import_entry(uuid: crate::asset::AssetUuid, entry: &ImportedAssetEntry) -> AssetId {
    asset_id_for_meta_entry(uuid, &entry.locator)
}

fn validate_import_entries(
    source_uri: &crate::asset::AssetUri,
    outcome: &AssetImportOutcome,
) -> Result<(), AssetImportError> {
    if outcome.entries.is_empty() {
        return Err(AssetImportError::Parse(format!(
            "importer did not return any asset entries for {source_uri}"
        )));
    }

    let mut labels = HashSet::new();
    let mut root_count = 0;
    for entry in &outcome.entries {
        if entry.locator.scheme() != source_uri.scheme()
            || entry.locator.path() != source_uri.path()
        {
            return Err(AssetImportError::Parse(format!(
                "imported asset entry locator {} does not belong to source {source_uri}",
                entry.locator
            )));
        }
        match entry.locator.label() {
            Some(label) => {
                if !labels.insert(label.to_string()) {
                    return Err(AssetImportError::DuplicateAssetLabel {
                        source_uri: source_uri.clone(),
                        label: label.to_string(),
                    });
                }
            }
            None => root_count += 1,
        }
    }
    if root_count != 1 {
        return Err(AssetImportError::Parse(format!(
            "importer returned {root_count} root entries for {source_uri}; expected exactly one"
        )));
    }
    Ok(())
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
