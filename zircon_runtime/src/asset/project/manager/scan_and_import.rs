use std::collections::{HashMap, HashSet};

use crate::core::framework::render::{
    shader_project_namespace_from_name, SHADER_IMPORT_PROJECT_NAMESPACE_SETTING,
};
use crate::core::resource::{ResourceDiagnostic, ResourceRecord, ResourceRegistryStaging};

use crate::asset::importer::stage_environment_ibl_source_with_parallel_executor;
use crate::asset::project::manager::durable_transaction::{
    commit_prepared_files, journal_directory, PreparedFileWrite, ProjectFileCommitOutcome,
    ProjectTransactionFault,
};
use crate::asset::watch::AssetChangeKind;
use crate::asset::{
    stage_environment_ibl_source, stage_external_source_cubemap_texture, AssetId,
    AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporterDescriptor, AssetKind,
    ImportedAsset,
};
use crate::core::runtime::tasks::TaskPool;

use super::ProjectManager;

mod dependency_resolution;
mod full_generation;
mod metadata;
mod projected_inventory;
mod shader_import_dependencies;
mod sources;
mod targeted;

pub(super) use shader_import_dependencies::ShaderImportDependencyIndex;
pub(crate) use targeted::PreparedTargetedGeneration;

pub(crate) enum PreparedWatchFileGeneration {
    Targeted(PreparedTargetedWatchChanges),
    Full(full_generation::PreparedFullProjectGeneration),
}

impl PreparedWatchFileGeneration {
    pub(crate) fn commit(self) -> Result<ProjectFileCommitOutcome, AssetImportError> {
        match self {
            Self::Targeted(prepared) => prepared.commit(),
            Self::Full(prepared) => prepared.commit(),
        }
    }
}

pub(crate) enum PreparedTargetedWatchChanges {
    Source(PreparedTargetedGeneration),
    Removal(PreparedTargetedWatchRemoval),
}

impl PreparedTargetedWatchChanges {
    pub(crate) fn commit(self) -> Result<ProjectFileCommitOutcome, AssetImportError> {
        match self {
            Self::Source(prepared) => prepared.commit(),
            Self::Removal(prepared) => prepared.commit(),
        }
    }
}

pub(crate) struct PreparedTargetedWatchRemoval {
    journal_directory: std::path::PathBuf,
    writes: Vec<PreparedFileWrite>,
}

impl PreparedTargetedWatchRemoval {
    fn commit(self) -> Result<ProjectFileCommitOutcome, AssetImportError> {
        commit_prepared_files(
            &self.journal_directory,
            self.writes,
            ProjectTransactionFault::None,
        )
    }
}

pub(super) fn stage_project_resource(
    registry: &mut ResourceRegistryStaging,
    record: ResourceRecord,
) -> Result<(), AssetImportError> {
    registry.stage_record(record).map(|_| ()).map_err(|error| {
        AssetImportError::Parse(format!("project resource catalog staging failed: {error}"))
    })
}

impl ProjectManager {
    pub fn scan_and_import(&mut self) -> Result<Vec<ResourceRecord>, AssetImportError> {
        let mut candidate = self.clone();
        let (imported, prepared) = candidate.prepare_full_generation(None)?;
        let outcome = prepared.commit()?;
        *self = candidate;
        outcome.ensure_durable()?;
        Ok(imported)
    }

    pub fn scan_and_import_watch_changes(
        &mut self,
        changes: &[crate::asset::watch::AssetChange],
    ) -> Result<Vec<ResourceRecord>, AssetImportError> {
        let mut candidate = self.clone();
        let use_incremental = Self::watch_changes_use_incremental_path(changes);
        let (updated_records, prepared) =
            candidate.prepare_watch_file_generation(changes, use_incremental)?;
        let outcome = prepared.commit()?;
        *self = candidate;
        outcome.ensure_durable()?;
        Ok(updated_records)
    }

    pub(crate) fn watch_changes_use_incremental_path(
        changes: &[crate::asset::watch::AssetChange],
    ) -> bool {
        matches!(
            changes,
            [change]
                if matches!(
                    change.kind,
                    AssetChangeKind::Added | AssetChangeKind::Modified | AssetChangeKind::Removed
                ) && change.previous_uri.is_none()
        )
    }

    #[cfg(test)]
    pub(crate) fn scan_and_import_watch_changes_with_registry_fault(
        &mut self,
        changes: &[crate::asset::watch::AssetChange],
        fault: crate::core::resource::io::AtomicWriteFault,
    ) -> Result<Vec<ResourceRecord>, AssetImportError> {
        if fault == crate::core::resource::io::AtomicWriteFault::None {
            self.scan_and_import_watch_changes(changes)
        } else {
            self.scan_and_import_with_commit_failure_before_registry(Some(changes))
        }
    }

    pub(crate) fn prepare_watch_file_generation(
        &mut self,
        changes: &[crate::asset::watch::AssetChange],
        use_incremental: bool,
    ) -> Result<(Vec<ResourceRecord>, PreparedWatchFileGeneration), AssetImportError> {
        if use_incremental {
            self.prepare_targeted_watch_changes(changes)
                .map(|(records, prepared)| {
                    (records, PreparedWatchFileGeneration::Targeted(prepared))
                })
        } else {
            self.prepare_full_generation(Some(changes))
                .map(|(records, prepared)| (records, PreparedWatchFileGeneration::Full(prepared)))
        }
    }

    pub(crate) fn prepare_targeted_watch_changes(
        &mut self,
        changes: &[crate::asset::watch::AssetChange],
    ) -> Result<(Vec<ResourceRecord>, PreparedTargetedWatchChanges), AssetImportError> {
        let [change] = changes else {
            unreachable!("only one complete source event enters the incremental watch path");
        };
        match change.kind {
            AssetChangeKind::Removed => {
                let (updated_records, prepared) =
                    self.prepare_targeted_watch_source_removal(&change.uri)?;
                Ok((
                    updated_records,
                    PreparedTargetedWatchChanges::Removal(prepared),
                ))
            }
            AssetChangeKind::Added | AssetChangeKind::Modified => {
                let source_path =
                    self.existing_or_primary_project_source_path_for_uri(&change.uri)?;
                let prepared = self.prepare_targeted_generation(&change.uri, &source_path)?;
                let updated_records = prepared
                    .imported()
                    .iter()
                    .chain(prepared.affected())
                    .cloned()
                    .collect();
                Ok((
                    updated_records,
                    PreparedTargetedWatchChanges::Source(prepared),
                ))
            }
            AssetChangeKind::Renamed => {
                unreachable!("rename events enter the complete reconciliation path")
            }
        }
    }

    fn prepare_targeted_watch_source_removal(
        &mut self,
        source: &crate::asset::AssetUri,
    ) -> Result<(Vec<ResourceRecord>, PreparedTargetedWatchRemoval), AssetImportError> {
        let removed_entries = self.asset_registry.source_entries(source);
        if removed_entries.is_empty() {
            return Ok((
                Vec::new(),
                PreparedTargetedWatchRemoval {
                    journal_directory: journal_directory(&self.paths),
                    writes: Vec::new(),
                },
            ));
        }
        let removed_ids = removed_entries
            .iter()
            .map(|entry| AssetId::from_asset_uuid(entry.uuid()))
            .collect::<HashSet<_>>();
        let mut registry = self.registry.begin_staging();
        for entry in &removed_entries {
            registry.stage_remove_locator(entry.path());
        }
        let (mut asset_registry, mut affected_uuids) =
            self.asset_registry.prepare_source_removal(source);
        let (shader_import_dependencies, shader_affected_ids) = self
            .shader_import_dependencies
            .prepare_source_replacement(&removed_ids, &[]);
        let dependency_changes = shader_affected_ids.into_iter().map(|id| {
            (
                id,
                self.shader_import_dependencies.dependency_locators(id),
                shader_import_dependencies.dependency_locators(id),
            )
        });
        affected_uuids.extend(asset_registry.retarget_runtime_dependency_paths(dependency_changes));
        self::targeted::refresh_runtime_dependency_closure(
            &mut registry,
            &asset_registry,
            &affected_uuids,
        )?;
        let catalog_updated_records = affected_uuids
            .iter()
            .filter_map(|uuid| registry.get(AssetId::from_asset_uuid(*uuid)).cloned())
            .collect::<Vec<_>>();
        let catalog_input_generation =
            crate::asset::project::ProjectCatalogInputGeneration::publish_targeted(
                &self.catalog_input_generation,
                self.paths.root(),
                &self.manifest,
                &self.package_assets,
                catalog_updated_records,
                HashMap::new(),
                removed_ids.iter().copied(),
            );
        let persisted = asset_registry.prepare_persistence(self.paths.registry_root())?;
        let mut affected = affected_uuids
            .into_iter()
            .filter_map(|uuid| registry.get(AssetId::from_asset_uuid(uuid)).cloned())
            .collect::<Vec<_>>();
        affected.sort_by(|left, right| left.primary_locator.cmp(&right.primary_locator));
        self.registry = registry.finish();
        self.asset_registry = asset_registry;
        self.shader_import_dependencies = shader_import_dependencies;
        self.catalog_input_generation = catalog_input_generation;
        Ok((
            affected,
            PreparedTargetedWatchRemoval {
                journal_directory: journal_directory(&self.paths),
                writes: vec![PreparedFileWrite::new(persisted.path, persisted.bytes)],
            },
        ))
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
}

fn stage_environment_ibl_import(
    context: &AssetImportContext,
    imported_asset: Option<&ImportedAsset>,
    cache_root: &std::path::Path,
    parallel_executor: Option<&TaskPool>,
) -> Result<(), AssetImportError> {
    let staging = match parallel_executor {
        Some(parallel_executor) => stage_environment_ibl_source_with_parallel_executor(
            context,
            cache_root,
            parallel_executor,
        ),
        None => stage_environment_ibl_source(context, cache_root),
    };
    staging.map_err(|error| {
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
