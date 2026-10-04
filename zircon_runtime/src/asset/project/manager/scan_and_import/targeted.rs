use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::asset::importer::canonical_import_input_digest;
use crate::asset::project::{
    AssetMetaDocument, AssetMetaEntry, PreviewState, ProjectCatalogInputGeneration,
    ProjectCatalogInputSource, ProjectPaths,
};
use crate::asset::registry::AssetRegistryDiagnostic;
use crate::asset::{
    AssetId, AssetImportContext, AssetImportError, AssetKind, AssetUri, ImportedAsset,
    ImportedAssetEntry,
};
use crate::core::resource::{
    ResourceDiagnostic, ResourceRecord, ResourceRegistryAssemblyExt, ResourceRegistryStaging,
    ResourceState,
};

use super::super::load_or_create_meta::verify_meta_precondition;
use super::metadata::{
    apply_importer_metadata, build_identity_for_import, clear_schema_migration_metadata,
    entry_uuid_for_import_entry, existing_entry_tags_for_source, existing_entry_uuids_for_source,
    failed_entries_for_source, validate_import_entries,
};
use super::sources::{
    materialize_compound_source_snapshot, source_asset_root_for_digest, source_digest_for_import,
    source_mtime_unix_ms_for_import, take_source_bytes_for_import, AssetImportSourceSnapshot,
};
use super::{stage_project_resource, ImportSourcePlan, ProjectManager};
use crate::asset::project::manager::durable_transaction::{
    commit_prepared_files, journal_directory, PreparedFileWrite, ProjectFileCommitOutcome,
    ProjectTransactionFault,
};

pub(crate) struct PreparedTargetedGeneration {
    journal_directory: PathBuf,
    meta_path: PathBuf,
    meta_precondition: Option<AssetMetaDocument>,
    writes: Vec<PreparedFileWrite>,
    registry_write: PreparedFileWrite,
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
        let _meta_write_guard = crate::asset::project::lock_meta_document_path(&self.meta_path)?;
        verify_meta_precondition(&self.meta_path, self.meta_precondition.as_ref())?;
        let mut writes = self.writes;
        writes.push(self.registry_write);
        commit_prepared_files(
            &self.journal_directory,
            writes,
            ProjectTransactionFault::None,
        )
    }

    #[cfg(test)]
    fn commit_with_fault(
        self,
        fault: ProjectTransactionFault,
    ) -> Result<ProjectFileCommitOutcome, AssetImportError> {
        let _meta_write_guard = crate::asset::project::lock_meta_document_path(&self.meta_path)?;
        verify_meta_precondition(&self.meta_path, self.meta_precondition.as_ref())?;
        let mut writes = self.writes;
        writes.push(self.registry_write);
        commit_prepared_files(&self.journal_directory, writes, fault)
    }
}

/// Collects targeted source preparations against one candidate project generation.
///
/// Every member owns its source/artifact/meta writes, while only the final candidate registry
/// write is retained. This is the durable boundary required by compound editor imports.
pub(crate) struct PreparedProjectImportBatch {
    journal_directory: PathBuf,
    meta_paths: Vec<PathBuf>,
    meta_preconditions: Vec<(PathBuf, Option<AssetMetaDocument>)>,
    writes: Vec<PreparedFileWrite>,
    registry_write: PreparedFileWrite,
    imported: Vec<ResourceRecord>,
    affected: Vec<ResourceRecord>,
    ready_payloads: Vec<(ResourceRecord, ImportedAsset)>,
}

impl PreparedProjectImportBatch {
    pub(crate) fn from_targeted_generations(
        generations: impl IntoIterator<Item = PreparedTargetedGeneration>,
    ) -> Result<Self, AssetImportError> {
        let mut generations = generations.into_iter();
        let first = generations
            .next()
            .ok_or(AssetImportError::EmptyProjectImportBatch)?;
        let mut batch = Self::from_generation(first);
        for generation in generations {
            batch.append(generation);
        }
        let mut unique_meta_paths = BTreeMap::new();
        for path in std::mem::take(&mut batch.meta_paths) {
            let identity = ProjectPaths::resolve_identity(&path)?;
            unique_meta_paths.entry(identity).or_insert(path);
        }
        batch.meta_paths = unique_meta_paths.into_values().collect();
        batch.append_prepared_writes(Vec::new())?;
        Ok(batch)
    }

    pub(crate) fn imported(&self) -> &[ResourceRecord] {
        &self.imported
    }

    pub(crate) fn affected(&self) -> &[ResourceRecord] {
        &self.affected
    }

    pub(crate) fn take_ready_payloads(&mut self) -> Vec<(ResourceRecord, ImportedAsset)> {
        std::mem::take(&mut self.ready_payloads)
    }

    pub(crate) fn append_source_writes(
        &mut self,
        writes: Vec<PreparedFileWrite>,
    ) -> Result<(), AssetImportError> {
        self.append_prepared_writes(writes)
    }

    pub(crate) fn commit(self) -> Result<ProjectFileCommitOutcome, AssetImportError> {
        let _meta_write_guards = crate::asset::project::lock_meta_document_paths(&self.meta_paths)?;
        for (path, expected) in &self.meta_preconditions {
            verify_meta_precondition(path, expected.as_ref())?;
        }
        let mut writes = self.writes;
        writes.push(self.registry_write);
        commit_prepared_files(
            &self.journal_directory,
            writes,
            ProjectTransactionFault::None,
        )
    }

    fn from_generation(generation: PreparedTargetedGeneration) -> Self {
        let PreparedTargetedGeneration {
            journal_directory,
            meta_path,
            meta_precondition,
            writes,
            registry_write,
            imported,
            affected,
            ready_payloads,
        } = generation;
        Self {
            journal_directory,
            meta_preconditions: vec![(meta_path.clone(), meta_precondition)],
            meta_paths: vec![meta_path],
            writes,
            registry_write,
            imported,
            affected,
            ready_payloads,
        }
    }

    fn append(&mut self, generation: PreparedTargetedGeneration) {
        let PreparedTargetedGeneration {
            journal_directory,
            meta_path,
            meta_precondition,
            writes,
            registry_write,
            imported,
            affected,
            ready_payloads,
        } = generation;
        debug_assert_eq!(self.journal_directory, journal_directory);
        self.meta_preconditions
            .push((meta_path.clone(), meta_precondition));
        self.meta_paths.push(meta_path);
        self.writes.extend(writes);
        self.registry_write = registry_write;
        self.imported.extend(imported);
        self.affected.extend(affected);
        self.ready_payloads.extend(ready_payloads);
    }

    fn append_prepared_writes(
        &mut self,
        writes: Vec<PreparedFileWrite>,
    ) -> Result<(), AssetImportError> {
        let existing_writes = std::mem::take(&mut self.writes);
        let mut prepared_paths = BTreeMap::new();
        super::append_prepared_file_writes(&mut self.writes, &mut prepared_paths, existing_writes)?;
        super::append_prepared_file_writes(&mut self.writes, &mut prepared_paths, writes)
    }
}

impl ProjectManager {
    pub(crate) fn prepare_targeted_import_batch(
        &mut self,
        sources: &[(AssetUri, PathBuf)],
    ) -> Result<PreparedProjectImportBatch, AssetImportError> {
        if sources.is_empty() {
            return Err(AssetImportError::EmptyProjectImportBatch);
        }
        let mut paths_by_uri = HashMap::with_capacity(sources.len());
        let mut generations = Vec::with_capacity(sources.len());
        for (uri, path) in sources {
            let source_uri = AssetUri::new(uri.scheme(), uri.path().to_string(), None)?;
            if let Some(previous) = paths_by_uri.insert(source_uri.clone(), path.clone()) {
                return Err(AssetImportError::DuplicateProjectAssetUri {
                    uri: source_uri,
                    first: previous,
                    second: path.clone(),
                });
            }
            generations.push(self.prepare_targeted_generation(&source_uri, path)?);
        }
        PreparedProjectImportBatch::from_targeted_generations(generations)
    }

    pub(crate) fn prepare_model_import_batch(
        &mut self,
        plan: ImportSourcePlan,
    ) -> Result<PreparedProjectImportBatch, AssetImportError> {
        const DEFAULT_PROJECT_MATERIAL_URI: &str = "res://materials/default.zmaterial";

        let source_uri = plan.source_uri().clone();
        let source_path = plan.source_path().to_path_buf();
        let source_snapshot = plan
            .source_bytes()
            .map(|source_bytes| AssetImportSourceSnapshot {
                source_bytes: source_bytes.to_vec(),
                source_mtime_unix_ms: plan.source_mtime_unix_ms().unwrap_or_default(),
                source_file_snapshots: plan.source_file_snapshots().clone(),
            });
        let material_uri = AssetUri::parse(DEFAULT_PROJECT_MATERIAL_URI)?;
        let material_path = self.existing_or_primary_project_source_path_for_uri(&material_uri)?;
        let model_generation = self.prepare_targeted_generation_with_source_snapshot(
            &source_uri,
            &source_path,
            source_snapshot,
            false,
        )?;
        let material_generation =
            self.prepare_targeted_generation(&material_uri, &material_path)?;
        let mut batch = PreparedProjectImportBatch::from_targeted_generations([
            model_generation,
            material_generation,
        ])?;
        batch.append_source_writes(plan.into_staged_writes())?;
        Ok(batch)
    }

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
        outcome.ensure_durable()?;
        *self = candidate;
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
        outcome.ensure_durable()?;
        *self = candidate;
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
        self.prepare_targeted_generation_with_source_snapshot(uri, indexed_path, None, false)
    }

    pub(crate) fn prepare_generated_source_generation(
        &mut self,
        uri: &AssetUri,
        indexed_path: &Path,
        source_bytes: Vec<u8>,
    ) -> Result<PreparedTargetedGeneration, AssetImportError> {
        let source_mtime_unix_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            .try_into()
            .unwrap_or(u64::MAX);
        self.prepare_targeted_generation_with_source_snapshot(
            uri,
            indexed_path,
            Some(AssetImportSourceSnapshot {
                source_bytes,
                source_mtime_unix_ms,
                source_file_snapshots: BTreeMap::new(),
            }),
            true,
        )
    }

    fn prepare_targeted_generation_with_source_snapshot(
        &mut self,
        uri: &AssetUri,
        indexed_path: &Path,
        source_snapshot: Option<AssetImportSourceSnapshot>,
        persist_source_snapshot: bool,
    ) -> Result<PreparedTargetedGeneration, AssetImportError> {
        let source = self.prepare_targeted_import_source(uri, indexed_path)?;
        let mut source = match source_snapshot {
            Some(snapshot) => source.with_source_snapshot(snapshot),
            None => source,
        };
        let replaced_ids = self
            .asset_registry
            .source_entries(&source.uri)
            .into_iter()
            .map(|entry| AssetId::from_asset_uuid(entry.uuid()))
            .collect::<HashSet<_>>();
        let source_snapshot_error = match materialize_compound_source_snapshot(&mut source) {
            Ok(()) => None,
            Err(error) if source.source_snapshot.is_none() => return Err(error),
            Err(error) => Some(error),
        };
        let source_bytes = take_source_bytes_for_import(&mut source)?;
        let source_mtime_unix_ms = source_mtime_unix_ms_for_import(&source)?;
        let source_file_snapshots = source.take_source_file_snapshots();
        let digest_primary = if source.compound_root.is_some() {
            &[][..]
        } else {
            source_bytes.as_slice()
        };
        let source_root = source_asset_root_for_digest(&source);
        let source_digest =
            source_digest_for_import(digest_primary, &source_file_snapshots, Some(&source_root));
        let build_input_digest = canonical_import_input_digest(
            &source.uri.to_string(),
            digest_primary,
            &source_file_snapshots,
            Some(&source_root),
        );
        let descriptor = self.importer.descriptor_for_source(&source.path).ok();
        let fallback_kind = descriptor
            .as_ref()
            .map(|descriptor| descriptor.output_kind)
            .unwrap_or(AssetKind::Data);
        let (mut meta, meta_precondition) = super::super::load_or_create_meta::load_or_create_meta(
            &source.meta_path,
            &source.uri,
            fallback_kind,
        )?;
        let previous_meta = meta.clone();
        meta.unit = source.unit;
        meta.included_files = source.included_files.clone();
        let import_settings =
            self.import_settings_for_source(&meta.import_settings, descriptor.as_ref());
        let build_identity = build_identity_for_import(
            &import_settings,
            &build_input_digest,
            descriptor.as_ref(),
            Some((&meta.importer_id, meta.importer_version)),
        );
        let config_hash = build_identity.action_key().to_string();
        if let Some(error) = source_snapshot_error {
            apply_importer_metadata(&mut meta, descriptor.as_ref());
            clear_schema_migration_metadata(&mut meta);
            meta.url = source.uri.clone();
            meta.asset_kind = fallback_kind;
            meta.unit = source.unit;
            meta.included_files = source.included_files.clone();
            meta.artifact_locator = None;
            meta.dependencies.clear();
            meta.entries =
                failed_entries_for_source(&previous_meta, meta.uuid, &source.uri, fallback_kind);
            meta.config_hash = config_hash.clone();
            meta.source_digest = source_digest.clone();
            meta.source_mtime_unix_ms = source_mtime_unix_ms;
            meta.preview_state = PreviewState::Error;

            let (mut asset_registry, mut affected_uuids) = self
                .asset_registry
                .prepare_source_replacement_generation(&mut meta)?;
            let root_asset_id = AssetId::from_asset_uuid(meta.uuid);
            let (shader_import_dependencies, shader_affected_ids) = self
                .shader_import_dependencies
                .prepare_source_replacement(&replaced_ids, std::iter::empty());
            let dependency_changes = shader_affected_ids.into_iter().map(|id| {
                (
                    id,
                    self.shader_import_dependencies.dependency_locators(id),
                    shader_import_dependencies.dependency_locators(id),
                )
            });
            affected_uuids
                .extend(asset_registry.retarget_runtime_dependency_paths(dependency_changes));

            let mut registry = self.registry.begin_staging();
            for previous in self.asset_registry.source_entries(&source.uri) {
                registry.stage_remove_locator(previous.path());
            }
            stage_project_resource(
                &mut registry,
                ResourceRecord::new(root_asset_id, fallback_kind, source.uri.clone())
                    .with_source_hash(source_digest)
                    .with_importer_id(meta.importer_id.clone())
                    .with_importer_version(meta.importer_version)
                    .with_config_hash(config_hash)
                    .with_state(ResourceState::Error)
                    .with_diagnostics(vec![ResourceDiagnostic::error(error.to_string())]),
            )?;
            refresh_runtime_dependency_closure(&mut registry, &asset_registry, &affected_uuids)?;
            let imported = vec![registry
                .get(root_asset_id)
                .cloned()
                .expect("failed source was staged")];
            let catalog_updated_records = std::iter::once(root_asset_id)
                .chain(affected_uuids.iter().copied().map(AssetId::from_asset_uuid))
                .filter_map(|id| registry.get(id).cloned())
                .collect::<Vec<_>>();
            let mut affected = affected_uuids
                .into_iter()
                .filter_map(|uuid| registry.get(AssetId::from_asset_uuid(uuid)).cloned())
                .collect::<Vec<_>>();
            affected.sort_by(|left, right| left.primary_locator.cmp(&right.primary_locator));
            let persisted = asset_registry.prepare_persistence(self.paths.registry_root())?;
            let registry_write = PreparedFileWrite::new(persisted.path, persisted.bytes);
            let writes = vec![PreparedFileWrite::new(
                source.meta_path.clone(),
                meta.to_pretty_bytes()?,
            )];
            self.registry = registry.finish();
            self.asset_registry = Arc::new(asset_registry);
            self.shader_import_dependencies = shader_import_dependencies;
            self.catalog_input_generation = ProjectCatalogInputGeneration::publish_targeted(
                &self.catalog_input_generation,
                self.paths.root(),
                &self.manifest,
                &self.package_assets,
                catalog_updated_records,
                HashMap::from([(
                    root_asset_id,
                    ProjectCatalogInputSource::new(
                        source.path,
                        source.meta_path.clone(),
                        meta,
                        source_mtime_unix_ms,
                        Vec::new(),
                        Vec::new(),
                    ),
                )]),
                replaced_ids.iter().copied(),
            );
            return Ok(PreparedTargetedGeneration {
                journal_directory: journal_directory(&self.paths),
                meta_path: source.meta_path,
                meta_precondition,
                writes,
                registry_write,
                imported,
                affected,
                ready_payloads: Vec::new(),
            });
        }
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
        .with_build_identity(build_identity)
        .with_source_file_snapshots(source_file_snapshots)
        .with_project_resolver(Arc::clone(&self.asset_registry), project_roots);
        let mut outcome = self.importer.import_context(&context)?;
        validate_import_entries(&source.uri, &outcome)?;
        let reference_repairs = outcome.reference_repairs.clone();
        let prepared_ibl_writes = super::prepare_environment_ibl_import(
            &context,
            outcome.root_entry().map(|entry| &entry.asset),
            self.paths.cache_root(),
            self.environment_ibl_parallel_executor.as_ref(),
        )?;
        let prepared_source_write = persist_source_snapshot
            .then(|| PreparedFileWrite::new(context.source_path.clone(), context.source_bytes));
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

        let mut writes = Vec::with_capacity(
            outcome.entries.len()
                + prepared_ibl_writes.len()
                + usize::from(prepared_source_write.is_some())
                + 2,
        );
        writes.extend(prepared_source_write);
        writes.extend(prepared_ibl_writes);
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

        let (shader_import_dependencies, shader_affected_ids) =
            self.shader_import_dependencies.prepare_source_replacement(
                &replaced_ids,
                ready_payloads
                    .iter()
                    .filter_map(|(record, asset)| match asset {
                        ImportedAsset::Shader(shader) => Some((record, shader)),
                        _ => None,
                    }),
            );
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
            reference_repairs,
        );

        writes.push(PreparedFileWrite::new(
            source.meta_path.clone(),
            meta.to_pretty_bytes()?,
        ));
        let persisted = asset_registry.prepare_persistence(self.paths.registry_root())?;
        let registry_write = PreparedFileWrite::new(persisted.path, persisted.bytes);
        self.registry = registry.finish();
        self.asset_registry = Arc::new(asset_registry);
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
            meta_precondition,
            writes,
            registry_write,
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

pub(crate) fn refresh_runtime_dependency_closure(
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

#[cfg(test)]
#[path = "targeted/tests/snapshot_failure_tests.rs"]
mod snapshot_failure_tests;
