use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use crate::asset::project::{
    AssetMetaDocument, AssetMetaEntry, PreviewState, ProjectCatalogInputGeneration,
    ProjectCatalogInputSource, ProjectGenerationObservation, ProjectGenerationPhase,
};
use crate::asset::registry::AssetRegistryIndex;
use crate::asset::watch::AssetChange;
use crate::asset::{
    AssetId, AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporterDescriptor,
    AssetKind, ImportedAsset,
};
use crate::core::resource::{ResourceDiagnostic, ResourceRecord, ResourceRegistry, ResourceState};

use super::dependency_resolution::{
    dependencies_for_entry, merge_handwritten_dependencies_into_meta, resolve_imported_dependencies,
};
use super::metadata::{
    apply_importer_metadata, asset_id_for_meta_entry, clear_schema_migration_metadata,
    config_hash_for_settings, entry_uuid_for_import_entry, existing_entry_tags_for_source,
    existing_entry_uuids_for_source, failed_entries_for_source, importer_contract_matches,
    remap_meta_entry_urls_to_source, validate_import_entries,
};
use super::projected_inventory::ProjectedMetaInventory;
use super::sources::{source_bytes_for_import, source_mtime_unix_ms_for_import, AssetImportSource};
use super::{stage_project_resource, ProjectManager, ShaderImportDependencyIndex};
use crate::asset::project::manager::durable_transaction::{
    commit_prepared_files, journal_directory, PreparedFileWrite, ProjectFileCommitOutcome,
    ProjectTransactionFault,
};

pub(crate) struct PreparedFullProjectGeneration {
    journal_directory: PathBuf,
    writes: Vec<PreparedFileWrite>,
    meta_paths: Vec<PathBuf>,
    meta_preconditions: Vec<(PathBuf, Option<AssetMetaDocument>)>,
    observation: ProjectGenerationObservation,
    #[cfg(test)]
    registry_write_index: usize,
}

impl PreparedFullProjectGeneration {
    pub(crate) fn commit(mut self) -> Result<ProjectFileCommitOutcome, AssetImportError> {
        let _phase = ProjectGenerationPhase::FileCommit.enter();
        let _meta_write_guards = crate::asset::project::lock_meta_document_paths(&self.meta_paths);
        verify_meta_preconditions(&self.meta_preconditions, &mut self.observation)?;
        let result = commit_prepared_files(
            &self.journal_directory,
            self.writes,
            ProjectTransactionFault::None,
        );
        if result.is_ok() {
            self.observation.mark_commit_succeeded();
        }
        result
    }

    #[cfg(test)]
    fn commit_with_failure_before_registry(
        self,
    ) -> Result<ProjectFileCommitOutcome, AssetImportError> {
        let registry_write_index = self.registry_write_index;
        self.commit_with_fault(ProjectTransactionFault::BeforeCommit(registry_write_index))
    }

    #[cfg(test)]
    fn commit_with_fault(
        mut self,
        fault: ProjectTransactionFault,
    ) -> Result<ProjectFileCommitOutcome, AssetImportError> {
        let _phase = ProjectGenerationPhase::FileCommit.enter();
        let _meta_write_guards = crate::asset::project::lock_meta_document_paths(&self.meta_paths);
        verify_meta_preconditions(&self.meta_preconditions, &mut self.observation)?;
        let result = commit_prepared_files(&self.journal_directory, self.writes, fault);
        if result.is_ok() {
            self.observation.mark_commit_succeeded();
        }
        result
    }
}

impl ProjectManager {
    pub(crate) fn prepare_full_generation(
        &mut self,
        watch_changes: Option<&[AssetChange]>,
    ) -> Result<(Vec<ResourceRecord>, PreparedFullProjectGeneration), AssetImportError> {
        let mut observation = ProjectGenerationObservation::new();
        let parallel_executor = self.environment_ibl_parallel_executor.clone();
        let sources = {
            let _phase = ProjectGenerationPhase::Discovery.enter();
            self.collect_import_sources(&mut observation)?
        };
        observation.record_sources(
            sources.len(),
            sources
                .iter()
                .map(|source| source.included_paths.len())
                .sum(),
        );
        let (mut inventory, duplicate_diagnostics, project_roots, import_registry) = {
            let _phase = ProjectGenerationPhase::MetadataProjection.enter();
            let mut inventory = ProjectedMetaInventory::load(self, &sources, &mut observation)?;
            let duplicate_diagnostics =
                inventory.normalize_duplicate_guids(&self.asset_registry, watch_changes);
            let project_roots = Arc::new(
                self.manifest
                    .asset_roots
                    .iter()
                    .cloned()
                    .zip(self.package_assets.project_roots().iter().cloned())
                    .collect::<Vec<_>>(),
            );
            let import_registry = Arc::new(AssetRegistryIndex::inspect_loaded_meta_document_refs(
                inventory.project_documents(),
            )?);
            (
                inventory,
                duplicate_diagnostics,
                project_roots,
                import_registry,
            )
        };

        let mut registry = ResourceRegistry::default().begin_staging();
        let mut dependencies_by_id = HashMap::new();
        let mut catalog_inputs = HashMap::new();
        let mut shader_import_paths = HashMap::new();
        let mut imported = Vec::with_capacity(sources.len());
        let mut writes = Vec::new();

        let _import_phase = ProjectGenerationPhase::Import.enter();
        for source in sources {
            let file = source.path.clone();
            let uri = source.uri.clone();
            let source_bytes = source_bytes_for_import(&source)?;
            observation.record_source_bytes(source_bytes.len());
            let source_digest = super::super::hash_bytes::hash_bytes(&source_bytes);
            let source_mtime_unix_ms = source_mtime_unix_ms_for_import(&source)?;
            let descriptor = self.importer.descriptor_for_source(&file).ok();
            let fallback_kind = descriptor
                .as_ref()
                .map(|descriptor| descriptor.output_kind)
                .unwrap_or(AssetKind::Data);
            let meta_path = source.meta_path.clone();
            let previous_meta = inventory.document(&meta_path).clone();
            let meta = inventory.document_mut(&meta_path);
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
                meta,
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
                    merge_handwritten_dependencies_into_meta(meta, asset);
                }
                super::stage_environment_ibl_import(
                    &import_context,
                    restored_root_asset.as_ref(),
                    self.paths.cache_root(),
                    parallel_executor.as_ref(),
                )?;
                let direct_references = restored_root_asset
                    .as_ref()
                    .map(ImportedAsset::direct_references)
                    .unwrap_or_default();
                catalog_inputs.insert(
                    root_asset_id,
                    ProjectCatalogInputSource::new(
                        file,
                        meta_path,
                        meta.clone(),
                        source_mtime_unix_ms,
                        direct_references,
                    ),
                );
                for record in metadata {
                    let asset_id = record.id();
                    dependencies_by_id.insert(
                        asset_id,
                        dependencies_for_entry(meta, record.primary_locator()),
                    );
                    stage_project_resource(&mut registry, record.clone())?;
                    imported.push(record);
                }
                observation.record_restored_source();
                continue;
            }

            let import_result = self.importer.import_context(&import_context);
            let (metadata, direct_references) = match import_result {
                Ok(outcome) => {
                    let validation = validate_import_entries(&uri, &outcome).and_then(|()| {
                        super::stage_environment_ibl_import(
                            &import_context,
                            outcome.root_entry().map(|entry| &entry.asset),
                            self.paths.cache_root(),
                            parallel_executor.as_ref(),
                        )
                    });
                    match validation {
                        Ok(()) => {
                            let mut outcome = outcome;
                            let direct_references = outcome
                                .root_entry()
                                .map(|entry| entry.asset.direct_references())
                                .unwrap_or_default();
                            super::append_shader_import_path_conflict_diagnostics(
                                &mut outcome,
                                &mut shader_import_paths,
                            );
                            let metadata = self.finish_successful_import(
                                &source,
                                meta,
                                &previous_meta,
                                source_digest.clone(),
                                source_mtime_unix_ms,
                                config_hash,
                                descriptor.as_ref(),
                                outcome,
                                &mut writes,
                                &mut observation,
                            )?;
                            observation.record_imported_source();
                            (metadata, direct_references)
                        }
                        Err(error) => {
                            let metadata = self.finish_failed_import(
                                &source,
                                meta,
                                &previous_meta,
                                source_digest.clone(),
                                source_mtime_unix_ms,
                                config_hash,
                                descriptor.as_ref(),
                                fallback_kind,
                                root_asset_id,
                                error,
                            )?;
                            observation.record_failed_source();
                            (metadata, Vec::new())
                        }
                    }
                }
                Err(error) => {
                    let metadata = self.finish_failed_import(
                        &source,
                        meta,
                        &previous_meta,
                        source_digest.clone(),
                        source_mtime_unix_ms,
                        config_hash,
                        descriptor.as_ref(),
                        fallback_kind,
                        root_asset_id,
                        error,
                    )?;
                    observation.record_failed_source();
                    (metadata, Vec::new())
                }
            };
            catalog_inputs.insert(
                root_asset_id,
                ProjectCatalogInputSource::new(
                    file,
                    meta_path,
                    meta.clone(),
                    source_mtime_unix_ms,
                    direct_references,
                ),
            );
            for record in metadata {
                let asset_id = record.id();
                dependencies_by_id.insert(
                    asset_id,
                    dependencies_for_entry(meta, record.primary_locator()),
                );
                stage_project_resource(&mut registry, record.clone())?;
                imported.push(record);
            }
        }
        drop(_import_phase);

        let shader_import_dependencies = {
            let _phase = ProjectGenerationPhase::DependencyProjection.enter();
            let shader_import_dependencies = ShaderImportDependencyIndex::from_artifacts(
                &self.artifact_store,
                &self.paths,
                &imported,
            )?;
            shader_import_dependencies.append_dependencies(&mut dependencies_by_id);
            resolve_imported_dependencies(&mut registry, &mut imported, &dependencies_by_id)?;
            shader_import_dependencies
        };

        let (mut asset_registry, catalog_input_generation) = {
            let _phase = ProjectGenerationPhase::RegistryProjection.enter();
            let mut asset_registry = self
                .asset_registry
                .rebuild_after_import_from_loaded(inventory.documents(), duplicate_diagnostics)?;
            asset_registry.reconcile_resource_dependencies(&registry, &dependencies_by_id);
            let catalog_input_generation = ProjectCatalogInputGeneration::publish(
                &self.catalog_input_generation,
                self.paths.root(),
                &self.manifest,
                &self.package_assets,
                registry.values().cloned(),
                catalog_inputs,
            );
            (asset_registry, catalog_input_generation)
        };

        let (meta_preconditions, meta_paths) = {
            let _phase = ProjectGenerationPhase::Serialize.enter();
            let meta_preconditions = inventory
                .preconditions()
                .map(|(path, document)| (path.clone(), document.cloned()))
                .collect::<Vec<_>>();
            let meta_paths = meta_preconditions
                .iter()
                .map(|(path, _)| path.clone())
                .collect::<Vec<_>>();
            let mut changed_metadata_count = 0;
            for (path, document) in inventory.changed_documents() {
                writes.push(PreparedFileWrite::new(
                    path.clone(),
                    document.to_pretty_bytes()?,
                ));
                changed_metadata_count += 1;
            }
            observation.record_changed_metadata(changed_metadata_count);
            let persisted = asset_registry.prepare_persistence(self.paths.registry_root())?;
            writes.push(PreparedFileWrite::new(persisted.path, persisted.bytes));
            (meta_preconditions, meta_paths)
        };
        #[cfg(test)]
        let registry_write_index = writes.len() - 1;
        observation.record_prepared_writes(
            writes.len(),
            writes.iter().fold(0_u64, |total, write| {
                total.saturating_add(u64::try_from(write.bytes.len()).unwrap_or(u64::MAX))
            }),
        );
        observation.mark_prepare_succeeded();

        self.registry = registry.finish();
        self.asset_registry = asset_registry;
        self.shader_import_dependencies = shader_import_dependencies;
        self.catalog_input_generation = catalog_input_generation;
        Ok((
            imported,
            PreparedFullProjectGeneration {
                journal_directory: journal_directory(&self.paths),
                writes,
                meta_paths,
                meta_preconditions,
                observation,
                #[cfg(test)]
                registry_write_index,
            },
        ))
    }

    fn restore_imported_artifact(
        &self,
        source: &AssetImportSource,
        meta: &mut AssetMetaDocument,
        previous_meta: &AssetMetaDocument,
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
            if self.artifact_store.read(&self.paths, artifact_uri).is_err() {
                return Ok(None);
            }
        }

        meta.url = uri.clone();
        if meta.asset_kind == AssetKind::Data && descriptor.is_some() {
            meta.asset_kind = fallback_kind;
        }
        meta.source_mtime_unix_ms = source_mtime_unix_ms;
        let _ = previous_meta;

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
        meta: &mut AssetMetaDocument,
        previous_meta: &AssetMetaDocument,
        source_digest: String,
        source_mtime_unix_ms: u64,
        config_hash: String,
        descriptor: Option<&AssetImporterDescriptor>,
        mut outcome: AssetImportOutcome,
        writes: &mut Vec<PreparedFileWrite>,
        observation: &mut ProjectGenerationObservation,
    ) -> Result<Vec<ResourceRecord>, AssetImportError> {
        crate::asset::registry::dependency_extractors::append_handwritten_dependencies(
            &mut outcome,
        );
        let uri = &source.uri;
        let root_entry = outcome.root_entry().ok_or_else(|| {
            AssetImportError::Parse(format!("importer did not return a root entry for {uri}"))
        })?;
        let kind = super::super::asset_kind::asset_kind(&root_entry.asset);
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
            let entry_kind = super::super::asset_kind::asset_kind(&entry.asset);
            let entry_uuid = entry_uuid_for_import_entry(meta.uuid, &existing_entry_uuids, &entry);
            let entry_asset_id = AssetId::from_asset_uuid(entry_uuid);
            let artifact_record =
                ResourceRecord::new(entry_asset_id, entry_kind, entry.locator.clone());
            let artifact =
                self.artifact_store
                    .prepare_write(&self.paths, &artifact_record, &entry.asset)?;
            observation.record_artifact(
                artifact.raw_bytes,
                artifact.compressed_bytes,
                artifact.chunk_count,
                artifact.payload.len(),
            );
            writes.push(PreparedFileWrite::new(
                artifact.artifact_path,
                artifact.payload,
            ));
            if entry.locator.label().is_none() {
                meta.artifact_locator = Some(artifact.locator.clone());
            }
            entries.push(AssetMetaEntry {
                uuid: entry_uuid,
                url: entry.locator.clone(),
                asset_kind: entry_kind,
                artifact_locator: Some(artifact.locator.clone()),
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
                    .with_artifact_locator(artifact.locator)
                    .with_state(ResourceState::Ready)
                    .with_diagnostics(entry.diagnostics),
            );
        }
        meta.entries = entries;
        Ok(records)
    }

    #[allow(clippy::too_many_arguments)]
    fn finish_failed_import(
        &self,
        source: &AssetImportSource,
        meta: &mut AssetMetaDocument,
        previous_meta: &AssetMetaDocument,
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

    #[cfg(test)]
    pub(crate) fn scan_and_import_with_commit_failure_before_registry(
        &mut self,
        watch_changes: Option<&[AssetChange]>,
    ) -> Result<Vec<ResourceRecord>, AssetImportError> {
        let mut candidate = self.clone();
        let (imported, prepared) = candidate.prepare_full_generation(watch_changes)?;
        let outcome = prepared.commit_with_failure_before_registry()?;
        *self = candidate;
        outcome.ensure_durable()?;
        Ok(imported)
    }

    #[cfg(test)]
    pub(crate) fn scan_and_import_with_staging_interruption(
        &mut self,
    ) -> Result<Vec<ResourceRecord>, AssetImportError> {
        self.scan_and_import_with_full_generation_fault(|_| {
            ProjectTransactionFault::CrashAfterStaging(0)
        })
    }

    #[cfg(test)]
    pub(crate) fn scan_and_import_with_target_replace_interruption(
        &mut self,
    ) -> Result<Vec<ResourceRecord>, AssetImportError> {
        self.scan_and_import_with_full_generation_fault(|_| {
            ProjectTransactionFault::CrashAfterTargetReplace(0)
        })
    }

    #[cfg(test)]
    pub(crate) fn scan_and_import_with_last_commit_interruption(
        &mut self,
    ) -> Result<Vec<ResourceRecord>, AssetImportError> {
        self.scan_and_import_with_full_generation_fault(ProjectTransactionFault::CrashAfterCommit)
    }

    #[cfg(test)]
    pub(crate) fn scan_and_import_with_terminal_interruption(
        &mut self,
        after_cleanup_transition: bool,
    ) -> Result<Vec<ResourceRecord>, AssetImportError> {
        self.scan_and_import_with_full_generation_fault(|_| {
            if after_cleanup_transition {
                ProjectTransactionFault::CrashAfterCleanup
            } else {
                ProjectTransactionFault::CrashAfterAllCommitted
            }
        })
    }

    #[cfg(test)]
    pub(crate) fn scan_and_import_with_commit_point_sync_failure(
        &mut self,
    ) -> Result<Vec<ResourceRecord>, AssetImportError> {
        self.scan_and_import_with_full_generation_fault(|_| {
            ProjectTransactionFault::FailCommitPointSync
        })
    }

    #[cfg(test)]
    fn scan_and_import_with_full_generation_fault(
        &mut self,
        select_fault: impl FnOnce(usize) -> ProjectTransactionFault,
    ) -> Result<Vec<ResourceRecord>, AssetImportError> {
        let mut candidate = self.clone();
        let (imported, prepared) = candidate.prepare_full_generation(None)?;
        let fault = select_fault(prepared.registry_write_index);
        let outcome = prepared.commit_with_fault(fault)?;
        *self = candidate;
        outcome.ensure_durable()?;
        Ok(imported)
    }
}

fn verify_meta_preconditions(
    preconditions: &[(PathBuf, Option<AssetMetaDocument>)],
    observation: &mut ProjectGenerationObservation,
) -> Result<(), AssetImportError> {
    for (path, expected) in preconditions {
        let current = match observation.load_metadata_document(path) {
            Ok(document) => Some(document),
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.into()),
        };
        if current.as_ref() != expected.as_ref() {
            return Err(AssetImportError::Parse(format!(
                "project metadata changed while full generation was prepared: {}",
                path.display()
            )));
        }
    }
    Ok(())
}
