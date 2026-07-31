use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;

use crate::asset::project::{AssetMetaEntry, PreviewState};
use crate::asset::registry::AssetRegistryDiagnostic;
use crate::asset::{
    AssetId, AssetImportContext, AssetImportError, AssetKind, AssetUri, ImportedAsset,
    ImportedAssetEntry,
};
use crate::core::resource::{ResourceDiagnostic, ResourceRecord, ResourceRegistry, ResourceState};

use super::metadata::{
    apply_importer_metadata, clear_schema_migration_metadata, config_hash_for_settings,
    entry_uuid_for_import_entry, existing_entry_tags_for_source, existing_entry_uuids_for_source,
    validate_import_entries,
};
use super::sources::{source_bytes_for_import, source_mtime_unix_ms_for_import};
use super::ProjectManager;
use crate::asset::project::manager::targeted_transaction::{
    commit_prepared_files, PreparedFileWrite, TargetedTransactionFault,
};

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
        self.import_targeted_source_with_fault(uri, indexed_path, TargetedTransactionFault::None)
    }

    #[cfg(test)]
    pub(crate) fn import_targeted_source_with_commit_failure(
        &mut self,
        uri: &AssetUri,
        indexed_path: &Path,
        file_index: usize,
    ) -> Result<Vec<ResourceRecord>, AssetImportError> {
        self.import_targeted_source_with_fault(
            uri,
            indexed_path,
            TargetedTransactionFault::BeforeCommit(file_index),
        )
        .map(|(imported, _, _)| imported)
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

    fn import_targeted_source_with_fault(
        &mut self,
        uri: &AssetUri,
        indexed_path: &Path,
        transaction_fault: TargetedTransactionFault,
    ) -> Result<
        (
            Vec<ResourceRecord>,
            Vec<ResourceRecord>,
            Vec<(ResourceRecord, ImportedAsset)>,
        ),
        AssetImportError,
    > {
        let source = self.prepare_targeted_import_source(uri, indexed_path)?;
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
        super::append_shader_import_path_conflict_diagnostics(&mut outcome, &mut HashMap::new());

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
        let (asset_registry, affected_uuids) = self
            .asset_registry
            .prepare_source_replacement_generation(&mut meta)?;

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
            writes.push(PreparedFileWrite {
                path: artifact.artifact_path,
                bytes: artifact.payload,
            });
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

        let mut registry = self.registry.clone();
        for previous in self.asset_registry.source_entries(&source.uri) {
            registry.remove_by_locator(previous.path());
        }
        for record in &imported {
            registry.upsert(record.clone());
        }
        refresh_runtime_dependency_closure(&mut registry, &asset_registry, &affected_uuids);
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

        writes.push(PreparedFileWrite {
            path: source.meta_path,
            bytes: meta.to_pretty_bytes()?,
        });
        let persisted = asset_registry.prepare_persistence(self.paths.registry_root())?;
        writes.push(PreparedFileWrite {
            path: persisted.path,
            bytes: persisted.bytes,
        });
        commit_prepared_files(writes, transaction_fault)?;

        self.registry = registry;
        self.asset_registry = asset_registry;
        let affected = affected_uuids
            .into_iter()
            .filter_map(|uuid| self.registry.get(AssetId::from_asset_uuid(uuid)).cloned())
            .collect();
        Ok((imported, affected, ready_payloads))
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

fn refresh_runtime_dependency_closure(
    registry: &mut ResourceRegistry,
    asset_registry: &crate::asset::registry::AssetRegistryIndex,
    affected_uuids: &HashSet<crate::asset::AssetUuid>,
) {
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
        registry.upsert(record);
    }
}
