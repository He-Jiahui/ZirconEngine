use std::collections::HashMap;

use crate::core::framework::render::{
    shader_project_namespace_from_name, SHADER_IMPORT_PROJECT_NAMESPACE_SETTING,
};
use crate::core::resource::{ResourceDiagnostic, ResourceRecord, ResourceRegistry, ResourceState};

use crate::asset::project::{AssetMetaEntry, PreviewState};
use crate::asset::registry::AssetRegistryIndex;
use crate::asset::{
    stage_environment_ibl_source, stage_external_source_cubemap_texture, AssetId,
    AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporterDescriptor, AssetKind,
    ImportedAsset,
};

use super::{
    asset_kind::asset_kind, hash_bytes::hash_bytes, load_or_create_meta::load_or_create_meta,
    ProjectManager,
};

mod dependency_resolution;
mod metadata;
mod shader_import_dependencies;
mod sources;

use self::dependency_resolution::{
    dependencies_for_entry, merge_handwritten_dependencies_into_meta, resolve_imported_dependencies,
};
use self::metadata::{
    apply_importer_metadata, asset_id_for_meta_entry, clear_schema_migration_metadata,
    config_hash_for_settings, entry_uuid_for_import_entry, existing_entry_tags_for_source,
    existing_entry_uuids_for_source, failed_entries_for_source, importer_contract_matches,
    remap_meta_entry_urls_to_source, validate_import_entries,
};
use self::sources::{source_bytes_for_import, source_mtime_unix_ms_for_import, AssetImportSource};

impl ProjectManager {
    pub fn scan_and_import(&mut self) -> Result<Vec<ResourceRecord>, AssetImportError> {
        self.scan_and_import_with_registry_update(
            None,
            crate::asset::project::meta_io::AtomicWriteFault::None,
        )
    }

    pub fn scan_and_import_watch_changes(
        &mut self,
        changes: &[crate::asset::watch::AssetChange],
    ) -> Result<Vec<ResourceRecord>, AssetImportError> {
        self.scan_and_import_with_registry_update(
            Some(changes),
            crate::asset::project::meta_io::AtomicWriteFault::None,
        )
    }

    #[cfg(test)]
    pub(crate) fn scan_and_import_watch_changes_with_registry_fault(
        &mut self,
        changes: &[crate::asset::watch::AssetChange],
        fault: crate::asset::project::meta_io::AtomicWriteFault,
    ) -> Result<Vec<ResourceRecord>, AssetImportError> {
        self.scan_and_import_with_registry_update(Some(changes), fault)
    }

    fn scan_and_import_with_registry_update(
        &mut self,
        watch_changes: Option<&[crate::asset::watch::AssetChange]>,
        registry_fault: crate::asset::project::meta_io::AtomicWriteFault,
    ) -> Result<Vec<ResourceRecord>, AssetImportError> {
        let sources = self.collect_import_sources()?;
        let asset_roots = self.registry_scan_roots();
        let registry_root = self.paths.registry_root().to_path_buf();
        // A moved sidecar still names its old URI until this preflight aligns it.
        let preflight_changes = self.prepare_reference_resolution_metadata(&sources)?;
        // Apply the authoritative sidecar rename before split remove/add watcher deltas.
        let mut identity_changes = preflight_changes;
        identity_changes.extend_from_slice(watch_changes.unwrap_or_default());
        // Normalize duplicated sidecar identities before resource ids are derived from them.
        let identity_changes =
            (!identity_changes.is_empty()).then_some(identity_changes.as_slice());
        let duplicate_diagnostics = self
            .asset_registry
            .prepare_duplicate_guids(&asset_roots, identity_changes)?;
        let project_roots = self
            .manifest
            .asset_roots
            .iter()
            .cloned()
            .zip(self.package_assets.project_roots().iter().cloned())
            .collect::<Vec<_>>();
        let import_registry = std::sync::Arc::new(AssetRegistryIndex::inspect_project(
            self.package_assets.project_roots(),
        )?);
        let project_roots = std::sync::Arc::new(project_roots);

        let mut registry = ResourceRegistry::default();
        let mut dependencies_by_id = HashMap::new();
        let mut shader_import_paths = HashMap::new();
        let mut imported = Vec::with_capacity(sources.len());

        for source in sources {
            let file = source.path.clone();
            let uri = source.uri.clone();
            let source_bytes = source_bytes_for_import(&source)?;
            let source_digest = hash_bytes(&source_bytes);
            let source_mtime_unix_ms = source_mtime_unix_ms_for_import(&source)?;
            let descriptor = self.importer.descriptor_for_source(&file).ok();
            let fallback_kind = descriptor
                .as_ref()
                .map(|descriptor| descriptor.output_kind)
                .unwrap_or(AssetKind::Data);
            let meta_path = source.meta_path.clone();
            let meta_exists = meta_path.exists();
            let mut meta = load_or_create_meta(&meta_path, &uri, fallback_kind)?;
            let previous_meta = meta.clone();
            meta.unit = source.unit;
            meta.included_files = source.included_files.clone();
            let import_settings =
                self.import_settings_for_source(&meta.import_settings, descriptor.as_ref());
            let config_hash = config_hash_for_settings(&import_settings);
            let root_asset_id = AssetId::from_asset_uuid(meta.uuid);
            let import_context =
                AssetImportContext::new(file.clone(), uri.clone(), source_bytes, import_settings)
                    .with_project_resolver(import_registry.clone(), project_roots.clone());

            if let Some(metadata) = self.restore_imported_artifact(
                &source,
                &mut meta,
                meta_exists,
                &previous_meta,
                source_digest.clone(),
                source_mtime_unix_ms,
                config_hash.clone(),
                descriptor.as_ref(),
                fallback_kind,
            )? {
                let restored_root_asset = meta
                    .entries
                    .iter()
                    .find(|entry| entry.url.label().is_none())
                    .and_then(|entry| entry.artifact_locator.as_ref())
                    .map(|locator| self.artifact_store.read(&self.paths, locator))
                    .transpose()?;
                if let Some(asset) = restored_root_asset.as_ref() {
                    merge_handwritten_dependencies_into_meta(&mut meta, asset);
                    if meta != previous_meta {
                        meta.save(&source.meta_path)?;
                    }
                }
                stage_environment_ibl_import(
                    &import_context,
                    restored_root_asset.as_ref(),
                    self.paths.cache_root(),
                )?;
                for record in metadata {
                    let asset_id = record.id();
                    dependencies_by_id.insert(
                        asset_id,
                        dependencies_for_entry(&meta, record.primary_locator()),
                    );
                    registry.upsert(record.clone());
                    imported.push(record);
                }
                continue;
            }

            let import_result = self.importer.import_context(&import_context);
            let metadata = match import_result {
                Ok(outcome) => {
                    let validation = validate_import_entries(&uri, &outcome).and_then(|()| {
                        stage_environment_ibl_import(
                            &import_context,
                            outcome.root_entry().map(|entry| &entry.asset),
                            self.paths.cache_root(),
                        )
                    });
                    match validation {
                        Ok(()) => {
                            let mut outcome = outcome;
                            append_shader_import_path_conflict_diagnostics(
                                &mut outcome,
                                &mut shader_import_paths,
                            );
                            self.finish_successful_import(
                                &source,
                                &mut meta,
                                meta_exists,
                                &previous_meta,
                                source_digest.clone(),
                                source_mtime_unix_ms,
                                config_hash,
                                descriptor.as_ref(),
                                outcome,
                            )?
                        }
                        Err(error) => self.finish_failed_import(
                            &source,
                            &mut meta,
                            meta_exists,
                            &previous_meta,
                            source_digest.clone(),
                            source_mtime_unix_ms,
                            config_hash,
                            descriptor.as_ref(),
                            fallback_kind,
                            root_asset_id,
                            error,
                        )?,
                    }
                }
                Err(error) => self.finish_failed_import(
                    &source,
                    &mut meta,
                    meta_exists,
                    &previous_meta,
                    source_digest.clone(),
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
                imported.push(record);
            }
        }

        shader_import_dependencies::append_shader_import_dependencies(
            &self.artifact_store,
            &self.paths,
            &imported,
            &mut dependencies_by_id,
        )?;
        resolve_imported_dependencies(&mut registry, &mut imported, &dependencies_by_id);

        let mut asset_registry = self.asset_registry.clone();
        if watch_changes.is_some() {
            asset_registry.apply_watch_changes_with_atomic_fault(
                &asset_roots,
                &registry_root,
                identity_changes.unwrap_or_default(),
                registry_fault,
            )?;
        } else {
            asset_registry.rebuild_after_import(&asset_roots, &registry_root)?;
        }
        asset_registry.replace_duplicate_diagnostics(duplicate_diagnostics);
        self.registry = registry;
        self.asset_registry = asset_registry;
        Ok(imported)
    }

    fn import_settings_for_source(
        &self,
        settings: &toml::Table,
        descriptor: Option<&AssetImporterDescriptor>,
    ) -> toml::Table {
        let mut settings = settings.clone();
        if descriptor.is_some_and(|descriptor| descriptor.allows_output_kind(AssetKind::Shader)) {
            settings.insert(
                SHADER_IMPORT_PROJECT_NAMESPACE_SETTING.to_string(),
                toml::Value::String(shader_project_namespace_from_name(&self.manifest.name)),
            );
        }
        settings
    }

    fn prepare_reference_resolution_metadata(
        &self,
        sources: &[AssetImportSource],
    ) -> Result<Vec<crate::asset::watch::AssetChange>, AssetImportError> {
        let mut changes = Vec::new();
        for source in sources {
            let previous = if source.meta_path.exists() {
                Some(crate::asset::project::AssetMetaDocument::load(
                    &source.meta_path,
                )?)
            } else {
                None
            };
            let fallback_kind = self
                .importer
                .descriptor_for_source(&source.path)
                .map(|descriptor| descriptor.output_kind)
                .unwrap_or(AssetKind::Data);
            let meta = load_or_create_meta(&source.meta_path, &source.uri, fallback_kind)?;
            if previous.as_ref() != Some(&meta) {
                if let Some(previous) = previous
                    .as_ref()
                    .filter(|previous| previous.url != meta.url)
                {
                    changes.push(crate::asset::watch::AssetChange::new(
                        crate::asset::watch::AssetChangeKind::Renamed,
                        meta.url.clone(),
                        Some(previous.url.clone()),
                    ));
                }
                meta.save(&source.meta_path)?;
            }
        }
        Ok(changes)
    }

    #[allow(clippy::too_many_arguments)]
    fn restore_imported_artifact(
        &self,
        source: &AssetImportSource,
        meta: &mut crate::asset::project::AssetMetaDocument,
        meta_exists: bool,
        previous_meta: &crate::asset::project::AssetMetaDocument,
        source_digest: String,
        source_mtime_unix_ms: u64,
        config_hash: String,
        descriptor: Option<&AssetImporterDescriptor>,
        fallback_kind: AssetKind,
    ) -> Result<Option<Vec<ResourceRecord>>, AssetImportError> {
        let uri = &source.uri;
        if meta.preview_state != PreviewState::Ready
            || meta.source_digest != source_digest
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
                uuid: meta.uuid,
                url: uri.clone(),
                asset_kind: meta.asset_kind,
                artifact_locator: Some(artifact_uri),
                dependencies: meta.dependencies.clone(),
                tags: meta.tags.clone(),
            }];
        }
        remap_meta_entry_urls_to_source(meta, uri);
        if let Some(root) = meta
            .entries
            .iter_mut()
            .find(|entry| entry.url.label().is_none())
        {
            root.tags = meta.tags.clone();
        }

        for entry in &meta.entries {
            let Some(artifact_uri) = &entry.artifact_locator else {
                return Ok(None);
            };
            if let Err(_error) = self.artifact_store.read(&self.paths, artifact_uri) {
                return Ok(None);
            }
        }

        meta.url = uri.clone();
        if meta.asset_kind == AssetKind::Data && descriptor.is_some() {
            meta.asset_kind = fallback_kind;
        }
        meta.source_mtime_unix_ms = source_mtime_unix_ms;
        if !meta_exists || meta != previous_meta {
            meta.save(&source.meta_path)?;
        }

        Ok(Some(
            meta.entries
                .iter()
                .map(|entry| {
                    let entry_asset_id = asset_id_for_meta_entry(entry);
                    let mut record =
                        ResourceRecord::new(entry_asset_id, entry.asset_kind, entry.url.clone())
                            .with_source_hash(source_digest.clone())
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
        source: &AssetImportSource,
        meta: &mut crate::asset::project::AssetMetaDocument,
        meta_exists: bool,
        previous_meta: &crate::asset::project::AssetMetaDocument,
        source_digest: String,
        source_mtime_unix_ms: u64,
        config_hash: String,
        descriptor: Option<&AssetImporterDescriptor>,
        mut outcome: AssetImportOutcome,
    ) -> Result<Vec<ResourceRecord>, AssetImportError> {
        crate::asset::registry::dependency_extractors::append_handwritten_dependencies(
            &mut outcome,
        );
        let uri = &source.uri;
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
        meta.url = uri.clone();
        meta.asset_kind = kind;
        meta.unit = source.unit;
        meta.included_files = source.included_files.clone();
        meta.artifact_locator = None;
        meta.dependencies = root_entry.dependencies.clone();
        meta.config_hash = config_hash.clone();
        meta.source_digest = source_digest.clone();
        meta.source_mtime_unix_ms = source_mtime_unix_ms;
        meta.preview_state = PreviewState::Ready;

        let mut entries = Vec::with_capacity(outcome.entries.len());
        let mut records = Vec::with_capacity(outcome.entries.len());
        let existing_entry_uuids = existing_entry_uuids_for_source(previous_meta, uri);
        let existing_entry_tags = existing_entry_tags_for_source(previous_meta, uri);
        for entry in outcome.entries {
            let entry_kind = asset_kind(&entry.asset);
            let entry_uuid = entry_uuid_for_import_entry(meta.uuid, &existing_entry_uuids, &entry);
            let entry_asset_id = AssetId::from_asset_uuid(entry_uuid);
            let artifact_record =
                ResourceRecord::new(entry_asset_id, entry_kind, entry.locator.clone());
            let artifact_uri =
                self.artifact_store
                    .write(&self.paths, &artifact_record, &entry.asset)?;
            if entry.locator.label().is_none() {
                meta.artifact_locator = Some(artifact_uri.clone());
            }
            entries.push(AssetMetaEntry {
                uuid: entry_uuid,
                url: entry.locator.clone(),
                asset_kind: entry_kind,
                artifact_locator: Some(artifact_uri.clone()),
                dependencies: entry.dependencies.clone(),
                tags: if entry.locator.label().is_none() {
                    meta.tags.clone()
                } else {
                    existing_entry_tags
                        .get(&entry.locator)
                        .cloned()
                        .unwrap_or_default()
                },
            });
            records.push(
                ResourceRecord::new(entry_asset_id, entry_kind, entry.locator)
                    .with_source_hash(source_digest.clone())
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
            meta.save(&source.meta_path)?;
        }

        Ok(records)
    }

    #[allow(clippy::too_many_arguments)]
    fn finish_failed_import(
        &self,
        source: &AssetImportSource,
        meta: &mut crate::asset::project::AssetMetaDocument,
        meta_exists: bool,
        previous_meta: &crate::asset::project::AssetMetaDocument,
        source_digest: String,
        source_mtime_unix_ms: u64,
        config_hash: String,
        descriptor: Option<&AssetImporterDescriptor>,
        kind: AssetKind,
        asset_id: AssetId,
        error: AssetImportError,
    ) -> Result<Vec<ResourceRecord>, AssetImportError> {
        let uri = &source.uri;
        apply_importer_metadata(meta, descriptor);
        clear_schema_migration_metadata(meta);
        meta.url = uri.clone();
        meta.asset_kind = kind;
        meta.unit = source.unit;
        meta.included_files = source.included_files.clone();
        meta.artifact_locator = None;
        meta.dependencies.clear();
        meta.entries = failed_entries_for_source(previous_meta, meta.uuid, uri, kind);
        meta.config_hash = config_hash.clone();
        meta.source_digest = source_digest.clone();
        meta.source_mtime_unix_ms = source_mtime_unix_ms;
        meta.preview_state = PreviewState::Error;
        if !meta_exists || meta != previous_meta {
            meta.save(&source.meta_path)?;
        }

        Ok(vec![ResourceRecord::new(asset_id, kind, uri.clone())
            .with_source_hash(source_digest)
            .with_importer_id(meta.importer_id.clone())
            .with_importer_version(meta.importer_version)
            .with_config_hash(config_hash)
            .with_state(ResourceState::Error)
            .with_diagnostics(vec![ResourceDiagnostic::error(
                error.to_string(),
            )])])
    }
}

fn stage_environment_ibl_import(
    context: &AssetImportContext,
    imported_asset: Option<&ImportedAsset>,
    cache_root: &std::path::Path,
) -> Result<(), AssetImportError> {
    stage_environment_ibl_source(context, cache_root).map_err(|error| {
        AssetImportError::Parse(format!(
            "stage environment IBL source {}: {error}",
            context.source_path.display()
        ))
    })?;
    if let Some(ImportedAsset::Texture(texture)) = imported_asset {
        stage_external_source_cubemap_texture(texture, cache_root).map_err(|error| {
            AssetImportError::Parse(format!(
                "stage environment IBL source {}: {error}",
                context.source_path.display()
            ))
        })?;
    }
    Ok(())
}

fn append_shader_import_path_conflict_diagnostics(
    outcome: &mut AssetImportOutcome,
    seen_import_paths: &mut HashMap<String, crate::asset::AssetUri>,
) {
    let Some(root_entry) = outcome
        .entries
        .iter_mut()
        .find(|entry| entry.locator.label().is_none())
    else {
        return;
    };
    let ImportedAsset::Shader(shader) = &root_entry.asset else {
        return;
    };
    let Some(import_path) = shader
        .import_path
        .as_ref()
        .filter(|path| !path.is_empty())
        .cloned()
    else {
        return;
    };
    if let Some(first_uri) = seen_import_paths.get(&import_path) {
        root_entry
            .diagnostics
            .push(ResourceDiagnostic::error(format!(
            "shader import_path `{import_path}` conflicts with already imported shader {first_uri}"
        )));
    } else {
        seen_import_paths.insert(import_path, root_entry.locator.clone());
    }
}
