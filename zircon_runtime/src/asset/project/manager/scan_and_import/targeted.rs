use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::asset::project::{
    AssetMetaEntry, PreviewState, ProjectCatalogInputGeneration, ProjectCatalogInputSource,
};
use crate::asset::registry::AssetRegistryDiagnostic;
use crate::asset::{
    AssetId, AssetImportContext, AssetImportError, AssetKind, AssetUri, ImportedAsset,
    ImportedAssetEntry,
};
use crate::core::resource::{
    ResourceDiagnostic, ResourceRecord, ResourceRegistryStaging, ResourceState,
};

use super::metadata::{
    apply_importer_metadata, clear_schema_migration_metadata, config_hash_for_settings,
    entry_uuid_for_import_entry, existing_entry_tags_for_source, existing_entry_uuids_for_source,
    validate_import_entries,
};
use super::sources::{source_bytes_for_import, source_mtime_unix_ms_for_import};
use super::{stage_project_resource, ProjectManager};
use crate::asset::project::manager::durable_transaction::{
    commit_prepared_files, journal_directory, PreparedFileWrite, ProjectFileCommitOutcome,
    ProjectTransactionFault,
};

pub(crate) struct PreparedTargetedGeneration {
    journal_directory: PathBuf,
    meta_path: PathBuf,
    writes: Vec<PreparedFileWrite>,
    imported: Vec<ResourceRecord>,
    affected: Vec<ResourceRecord>,
    ready_payloads: Vec<(ResourceRecord, ImportedAsset)>,
}

impl PreparedTargetedGeneration {
    pub(crate) fn imported(&self) -> &[ResourceRecord] {
        &self.imported
    }

    pub(crate) fn affected(&self) -> &[ResourceRecord] {
        &self.affected
    }

    pub(crate) fn take_ready_payloads(&mut self) -> Vec<(ResourceRecord, ImportedAsset)> {
        std::mem::take(&mut self.ready_payloads)
    }

    pub(crate) fn commit(self) -> Result<ProjectFileCommitOutcome, AssetImportError> {
        let _meta_write_guard = crate::asset::project::lock_meta_document_path(&self.meta_path);
        commit_prepared_files(
            &self.journal_directory,
            self.writes,
            ProjectTransactionFault::None,
        )
    }

    #[cfg(test)]
    fn commit_with_fault(
        self,
        fault: ProjectTransactionFault,
    ) -> Result<ProjectFileCommitOutcome, AssetImportError> {
        let _meta_write_guard = crate::asset::project::lock_meta_document_path(&self.meta_path);
        commit_prepared_files(&self.journal_directory, self.writes, fault)
    }
}

impl ProjectManager {
    pub(crate) fn import_targeted_source(
        &mut self,
        uri: &AssetUri,
        indexed_path: &Path,
    ) -> Result<Vec<ResourceRecord>, AssetImportError> {
        self.import_targeted_generation(uri, indexed_path)
            .map(|(imported, _, _)| imported)
    }

    pub(crate) fn import_targeted_generation(
        &mut self,
        uri: &AssetUri,
        indexed_path: &Path,
    ) -> Result<
        (
            Vec<ResourceRecord>,
            Vec<ResourceRecord>,
            Vec<(ResourceRecord, ImportedAsset)>,
        ),
        AssetImportError,
    > {
        let mut candidate = self.clone();
        let mut prepared = candidate.prepare_targeted_generation(uri, indexed_path)?;
        let imported = prepared.imported.clone();
        let affected = prepared.affected.clone();
        let ready_payloads = prepared.take_ready_payloads();
        let outcome = prepared.commit()?;
        *self = candidate;
        outcome.ensure_durable()?;
        Ok((imported, affected, ready_payloads))
    }

    #[cfg(test)]
    pub(crate) fn import_targeted_source_with_commit_failure(
        &mut self,
        uri: &AssetUri,
        indexed_path: &Path,
        file_index: usize,
    ) -> Result<Vec<ResourceRecord>, AssetImportError> {
        let mut candidate = self.clone();
        let prepared = candidate.prepare_targeted_generation(uri, indexed_path)?;
        let imported = prepared.imported.clone();
        let outcome =
            prepared.commit_with_fault(ProjectTransactionFault::BeforeCommit(file_index))?;
        *self = candidate;
        outcome.ensure_durable()?;
        Ok(imported)
    }

    #[cfg(test)]
    pub(crate) fn validate_targeted_source_topology(
        &self,
        uri: &AssetUri,
        indexed_path: &Path,
    ) -> Result<(), AssetImportError> {
        self.prepare_targeted_import_source(uri, indexed_path)
            .map(|_| ())
    }

    pub(crate) fn prepare_targeted_generation(
        &mut self,
        uri: &AssetUri,
        indexed_path: &Path,
    ) -> Result<PreparedTargetedGeneration, AssetImportError> {
        let source = self.prepare_targeted_import_source(uri, indexed_path)?;
        let replaced_ids = self
            .asset_registry
            .source_entries(&source.uri)
            .into_iter()
            .map(|entry| AssetId::from_asset_uuid(entry.uuid()))
            .collect::<HashSet<_>>();
        let source_bytes = source_bytes_for_import(&source)?;
        let source_digest = super::super::hash_bytes::hash_bytes(&source_bytes);
        let source_mtime_unix_ms = source_mtime_unix_ms_for_import(&source)?;
        let descriptor = self.importer.descriptor_for_source(&source.path).ok();
        let fallback_kind = descriptor
            .as_ref()
            .map(|descriptor| descriptor.output_kind)
            .unwrap_or(AssetKind::Data);
        let mut meta = super::super::load_or_create_meta::load_or_create_meta(
            &source.meta_path,
            &source.uri,
            fallback_kind,
        )?;
        let previous_meta = meta.clone();
        meta.unit = source.unit;
        meta.included_files = source.included_files.clone();
        let import_settings =
            self.import_settings_for_source(&meta.import_settings, descriptor.as_ref());
        let config_hash = config_hash_for_settings(&import_settings);
        let project_roots = Arc::new(
            self.manifest
                .asset_roots
                .iter()
                .cloned()
                .zip(self.package_assets.project_roots().iter().cloned())
                .collect::<Vec<_>>(),
        );
        let context = AssetImportContext::new(
            source.path.clone(),
            source.uri.clone(),
            source_bytes,
            import_settings,
        )
        .with_project_resolver(Arc::new(self.asset_registry.clone()), project_roots);
        let mut outcome = self.importer.import_context(&context)?;
        validate_import_entries(&source.uri, &outcome)?;
        super::stage_environment_ibl_import(
            &context,
            outcome.root_entry().map(|entry| &entry.asset),
            self.paths.cache_root(),
            self.environment_ibl_parallel_executor.as_ref(),
        )?;
        crate::asset::registry::dependency_extractors::append_handwritten_dependencies(
            &mut outcome,
        );
        let mut shader_import_paths = self
            .shader_import_dependencies
            .import_path_owners_excluding(&replaced_ids);
        super::append_shader_import_path_conflict_diagnostics(
            &mut outcome,
            &mut shader_import_paths,
        );

        prepare_meta_entries(
            &mut meta,
            &previous_meta,
            &source,
            source_digest.clone(),
            source_mtime_unix_ms,
            config_hash.clone(),
            descriptor.as_ref(),
            &outcome.entries,
        )?;
        let (mut asset_registry, mut affected_uuids) = self
            .asset_registry
            .prepare_source_replacement_generation(&mut meta)?;
        // Registry normalization can remint a colliding root UUID; the catalog key follows it.
        let root_asset_id = AssetId::from_asset_uuid(meta.uuid);

        let mut writes = Vec::with_capacity(outcome.entries.len() + 2);
        let mut imported = Vec::with_capacity(outcome.entries.len());
        let mut ready_payloads = Vec::with_capacity(outcome.entries.len());
        for (entry, meta_entry) in outcome.entries.into_iter().zip(&mut meta.entries) {
            let entry_kind = super::super::asset_kind::asset_kind(&entry.asset);
            let asset_id = AssetId::from_asset_uuid(meta_entry.uuid);
            let artifact_record = ResourceRecord::new(asset_id, entry_kind, entry.locator.clone());
            let artifact =
                self.artifact_store
                    .prepare_write(&self.paths, &artifact_record, &entry.asset)?;
            meta_entry.artifact_locator = Some(artifact.locator.clone());
            if entry.locator.label().is_none() {
                meta.artifact_locator = Some(artifact.locator.clone());
            }
            writes.push(PreparedFileWrite::new(
                artifact.artifact_path,
                artifact.payload,
            ));
            let record = ResourceRecord::new(asset_id, entry_kind, entry.locator)
                .with_source_hash(source_digest.clone())
                .with_importer_id(meta.importer_id.clone())
                .with_importer_version(meta.importer_version)
                .with_config_hash(config_hash.clone())
                .with_artifact_locator(artifact.locator)
                .with_state(ResourceState::Ready)
                .with_diagnostics(entry.diagnostics);
            ready_payloads.push((record.clone(), entry.asset));
            imported.push(record);
        }

        let (shader_import_dependencies, shader_affected_ids) = self
            .shader_import_dependencies
            .prepare_source_replacement(&replaced_ids, &ready_payloads);
        let dependency_changes = shader_affected_ids.into_iter().map(|id| {
            (
                id,
                self.shader_import_dependencies.dependency_locators(id),
                shader_import_dependencies.dependency_locators(id),
            )
        });
        affected_uuids.extend(asset_registry.retarget_runtime_dependency_paths(dependency_changes));

        let mut registry = self.registry.begin_staging();
        for previous in self.asset_registry.source_entries(&source.uri) {
            registry.stage_remove_locator(previous.path());
        }
        for record in &imported {
            stage_project_resource(&mut registry, record.clone())?;
        }
        refresh_runtime_dependency_closure(&mut registry, &asset_registry, &affected_uuids)?;
        for record in &mut imported {
            if let Some(resolved) = registry.get(record.id()).cloned() {
                *record = resolved;
            }
        }
        for (record, _) in &mut ready_payloads {
            if let Some(resolved) = registry.get(record.id()).cloned() {
                *record = resolved;
            }
        }
        let root_direct_references = ready_payloads
            .iter()
            .find(|(record, _)| record.primary_locator().label().is_none())
            .map(|(_, asset)| asset.direct_references())
            .unwrap_or_default();
        let catalog_input = ProjectCatalogInputSource::new(
            source.path.clone(),
            source.meta_path.clone(),
            meta.clone(),
            source_mtime_unix_ms,
            root_direct_references,
        );

        writes.push(PreparedFileWrite::new(
            source.meta_path.clone(),
            meta.to_pretty_bytes()?,
        ));
        let persisted = asset_registry.prepare_persistence(self.paths.registry_root())?;
        writes.push(PreparedFileWrite::new(persisted.path, persisted.bytes));
        self.registry = registry.finish();
        self.asset_registry = asset_registry;
        self.shader_import_dependencies = shader_import_dependencies;
        let catalog_updated_records = std::iter::once(root_asset_id)
            .chain(affected_uuids.iter().copied().map(AssetId::from_asset_uuid))
            .filter_map(|id| self.registry.get(id).cloned())
            .collect::<Vec<_>>();
        self.catalog_input_generation = ProjectCatalogInputGeneration::publish_targeted(
            &self.catalog_input_generation,
            self.paths.root(),
            &self.manifest,
            &self.package_assets,
            catalog_updated_records,
            HashMap::from([(root_asset_id, catalog_input)]),
            replaced_ids.iter().copied(),
        );
        let mut affected = affected_uuids
            .into_iter()
            .filter_map(|uuid| self.registry.get(AssetId::from_asset_uuid(uuid)).cloned())
            .collect::<Vec<_>>();
        affected.sort_by(|left, right| left.primary_locator.cmp(&right.primary_locator));
        Ok(PreparedTargetedGeneration {
            journal_directory: journal_directory(&self.paths),
            meta_path: source.meta_path,
            writes,
            imported,
            affected,
            ready_payloads,
        })
    }
}

#[allow(clippy::too_many_arguments)]
fn prepare_meta_entries(
    meta: &mut crate::asset::project::AssetMetaDocument,
    previous_meta: &crate::asset::project::AssetMetaDocument,
    source: &super::sources::AssetImportSource,
    source_digest: String,
    source_mtime_unix_ms: u64,
    config_hash: String,
    descriptor: Option<&crate::asset::AssetImporterDescriptor>,
    entries: &[ImportedAssetEntry],
) -> Result<(), AssetImportError> {
    let root = entries
        .iter()
        .find(|entry| entry.locator.label().is_none())
        .ok_or_else(|| {
            AssetImportError::Parse(format!(
                "importer did not return a root entry for {}",
                source.uri
            ))
        })?;
    let kind = super::super::asset_kind::asset_kind(&root.asset);
    apply_importer_metadata(meta, descriptor);
    if let Some(migration) = &root.migration_report {
        meta.source_schema_version = migration.source_schema_version;
        meta.target_schema_version = Some(migration.target_schema_version);
        meta.migration_summary = migration.summary.clone();
    } else {
        clear_schema_migration_metadata(meta);
    }
    meta.url = source.uri.clone();
    meta.asset_kind = kind;
    meta.unit = source.unit;
    meta.included_files = source.included_files.clone();
    meta.artifact_locator = None;
    meta.dependencies = root.dependencies.clone();
    meta.config_hash = config_hash;
    meta.source_digest = source_digest;
    meta.source_mtime_unix_ms = source_mtime_unix_ms;
    meta.preview_state = PreviewState::Ready;

    let existing_uuids = existing_entry_uuids_for_source(previous_meta, &source.uri);
    let existing_tags = existing_entry_tags_for_source(previous_meta, &source.uri);
    meta.entries = entries
        .iter()
        .map(|entry| {
            let uuid = entry_uuid_for_import_entry(meta.uuid, &existing_uuids, entry);
            AssetMetaEntry {
                uuid,
                url: entry.locator.clone(),
                asset_kind: super::super::asset_kind::asset_kind(&entry.asset),
                artifact_locator: None,
                dependencies: entry.dependencies.clone(),
                tags: if entry.locator.label().is_none() {
                    meta.tags.clone()
                } else {
                    existing_tags
                        .get(&entry.locator)
                        .cloned()
                        .unwrap_or_default()
                },
            }
        })
        .collect();
    Ok(())
}

pub(super) fn refresh_runtime_dependency_closure(
    registry: &mut ResourceRegistryStaging,
    asset_registry: &crate::asset::registry::AssetRegistryIndex,
    affected_uuids: &HashSet<crate::asset::AssetUuid>,
) -> Result<(), AssetImportError> {
    const UNRESOLVED_PREFIX: &str = "unresolved asset dependency ";
    for uuid in affected_uuids {
        let id = AssetId::from_asset_uuid(*uuid);
        let Some(mut record) = registry.get(id).cloned() else {
            continue;
        };
        record.dependency_ids = asset_registry
            .get_dependencies_by_uuid(*uuid)
            .into_iter()
            .map(AssetId::from_asset_uuid)
            .collect();
        record
            .diagnostics
            .retain(|diagnostic| !diagnostic.message.starts_with(UNRESOLVED_PREFIX));
        record.diagnostics.extend(
            asset_registry
                .diagnostics()
                .iter()
                .filter_map(|diagnostic| match diagnostic {
                    AssetRegistryDiagnostic::UnresolvedDependency { owner, path }
                        if owner == uuid =>
                    {
                        Some(ResourceDiagnostic::error(format!(
                            "{UNRESOLVED_PREFIX}{path}"
                        )))
                    }
                    _ => None,
                }),
        );
        stage_project_resource(registry, record)?;
    }
    Ok(())
}
