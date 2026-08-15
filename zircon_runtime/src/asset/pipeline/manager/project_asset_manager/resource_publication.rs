use crate::asset::project::ProjectGenerationPhase;
use crate::asset::{AssetUri, ImportedAsset, ProjectManager};
use crate::core::resource::{ResourceMutationBatch, ResourceRecord};
use crate::core::CoreError;

use super::super::errors::asset_error;
use super::super::resource_sync::register_project_resource;
use super::project_asset_manager::ProjectSourcePathIndex;
use super::ProjectAssetManager;

pub(in crate::asset::pipeline::manager) struct PreparedProjectResourceSync {
    source_paths: ProjectSourcePathIndex,
    resources: Vec<PreparedProjectResource>,
}

pub(in crate::asset::pipeline::manager) struct PreparedTargetedProjectResourceSync {
    source_uri: AssetUri,
    source_path: std::path::PathBuf,
    removed_locators: Vec<AssetUri>,
    resources: Vec<PreparedProjectResource>,
    record_updates: Vec<ResourceRecord>,
}

pub(super) struct PreparedIncrementalProjectResourceSync {
    removed_locators: Vec<AssetUri>,
    source_path_removals: Vec<AssetUri>,
    source_path_updates: Vec<(AssetUri, std::path::PathBuf)>,
    pub(super) record_updates: Vec<ResourceRecord>,
}

pub(super) enum PreparedWatchProjectResourceSync {
    Reconciliation(PreparedProjectResourceSync),
    Incremental(PreparedIncrementalProjectResourceSync),
}

enum PreparedProjectResource {
    Record(ResourceRecord),
    Ready(ResourceRecord, ImportedAsset),
}

fn append_prepared_resources(
    mut batch: ResourceMutationBatch,
    resources: impl IntoIterator<Item = PreparedProjectResource>,
) -> ResourceMutationBatch {
    for resource in resources {
        batch = match resource {
            PreparedProjectResource::Record(metadata) => batch.upsert_lazy(metadata),
            PreparedProjectResource::Ready(metadata, imported) => {
                register_project_resource(batch, metadata, imported)
            }
        };
    }
    batch
}

impl ProjectAssetManager {
    pub(in crate::asset::pipeline::manager) fn prepare_project_resource_sync(
        &self,
        project: &ProjectManager,
    ) -> Result<PreparedProjectResourceSync, CoreError> {
        let _phase = ProjectGenerationPhase::ResourceProjection.enter();
        let source_paths = Self::build_project_source_paths(project)?;
        let resources = project
            .registry()
            .values()
            .cloned()
            .map(PreparedProjectResource::Record)
            .collect();
        Ok(PreparedProjectResourceSync {
            source_paths,
            resources,
        })
    }

    pub(in crate::asset::pipeline::manager) fn commit_project_resource_sync<T>(
        &self,
        prepared: PreparedProjectResourceSync,
        batch: ResourceMutationBatch,
        commit_files: impl FnOnce() -> Result<T, CoreError>,
        commit_project_state: impl FnOnce(),
    ) -> Result<T, CoreError> {
        let batch = append_prepared_resources(batch, prepared.resources);
        self.commit_resource_batch_after_dependencies(batch, || {
            let outcome = commit_files()?;
            {
                let _phase = ProjectGenerationPhase::ProjectInstall.enter();
                *self.project_source_paths_write() = prepared.source_paths;
                commit_project_state();
            }
            Ok(outcome)
        })
    }

    pub(super) fn prepare_incremental_project_resource_sync(
        &self,
        project: &ProjectManager,
        previous_source_records: &[ResourceRecord],
        updated_records: Vec<ResourceRecord>,
    ) -> PreparedIncrementalProjectResourceSync {
        let _phase = ProjectGenerationPhase::ResourceProjection.enter();
        let mut records_by_id = std::collections::HashMap::new();
        for record in updated_records {
            records_by_id.insert(record.id(), record);
        }
        let removed_locators = previous_source_records
            .iter()
            .filter(|previous| {
                project
                    .registry()
                    .get(previous.id())
                    .is_none_or(|current| current.primary_locator != previous.primary_locator)
            })
            .map(|record| record.primary_locator.clone())
            .collect::<Vec<_>>();
        let mut source_path_updates = std::collections::HashMap::new();
        for record in records_by_id.values() {
            let source_uri = AssetUri::new(
                record.primary_locator.scheme(),
                record.primary_locator.path().to_string(),
                None,
            )
            .expect("a parsed resource locator remains valid when its label is removed");
            if let Ok(source_path) = project.source_path_for_uri(&source_uri) {
                source_path_updates.insert(source_uri, source_path);
            }
        }
        let source_path_removals = previous_source_records
            .iter()
            .map(|record| {
                AssetUri::new(
                    record.primary_locator.scheme(),
                    record.primary_locator.path().to_string(),
                    None,
                )
                .expect("a parsed resource locator remains valid when its label is removed")
            })
            .filter(|source_uri| project.source_resource_records(source_uri).is_empty())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        PreparedIncrementalProjectResourceSync {
            removed_locators,
            source_path_removals,
            source_path_updates: source_path_updates.into_iter().collect(),
            record_updates: records_by_id.into_values().collect(),
        }
    }

    pub(super) fn commit_incremental_project_resource_sync<T>(
        &self,
        prepared: PreparedIncrementalProjectResourceSync,
        commit_files: impl FnOnce() -> Result<T, CoreError>,
        commit_project_state: impl FnOnce(),
    ) -> Result<T, CoreError> {
        let mut batch = ResourceMutationBatch::new();
        for locator in prepared.removed_locators {
            batch = batch.remove(locator);
        }
        for record in prepared.record_updates {
            batch = batch.upsert_lazy(record);
        }
        self.commit_resource_batch_after_dependencies(batch, || {
            let outcome = commit_files()?;
            {
                let _phase = ProjectGenerationPhase::ProjectInstall.enter();
                let mut source_paths = self.project_source_paths_write();
                for source_uri in prepared.source_path_removals {
                    let remove_scheme =
                        source_paths
                            .get_mut(&source_uri.scheme())
                            .is_some_and(|paths| {
                                paths.remove(source_uri.path());
                                paths.is_empty()
                            });
                    if remove_scheme {
                        source_paths.remove(&source_uri.scheme());
                    }
                }
                for (source_uri, source_path) in prepared.source_path_updates {
                    source_paths
                        .entry(source_uri.scheme())
                        .or_default()
                        .insert(source_uri.path().to_string(), source_path);
                }
                commit_project_state();
            }
            Ok(outcome)
        })
    }

    pub(in crate::asset::pipeline::manager) fn prepare_targeted_project_resource_sync(
        &self,
        project: &ProjectManager,
        source_uri: &AssetUri,
        source_path: std::path::PathBuf,
        previous_source_records: &[ResourceRecord],
        imported: &[ResourceRecord],
        affected: &[ResourceRecord],
        ready_payloads: Vec<(ResourceRecord, ImportedAsset)>,
    ) -> PreparedTargetedProjectResourceSync {
        let _phase = ProjectGenerationPhase::ResourceProjection.enter();
        let current_source_records = project.source_resource_records(source_uri);
        let current_locators = current_source_records
            .iter()
            .map(|record| record.primary_locator().clone())
            .collect::<std::collections::HashSet<_>>();
        let removed_locators = previous_source_records
            .iter()
            .map(|record| record.primary_locator().clone())
            .filter(|locator| !current_locators.contains(locator))
            .collect();
        let imported_ids = imported
            .iter()
            .map(ResourceRecord::id)
            .collect::<std::collections::HashSet<_>>();
        let resources = ready_payloads
            .into_iter()
            .map(|(metadata, payload)| PreparedProjectResource::Ready(metadata, payload))
            .collect();
        let record_updates = affected
            .iter()
            .filter(|record| !imported_ids.contains(&record.id()))
            .cloned()
            .collect();
        PreparedTargetedProjectResourceSync {
            source_uri: AssetUri::new(source_uri.scheme(), source_uri.path().to_string(), None)
                .expect("a parsed source URI remains valid when its label is removed"),
            source_path,
            removed_locators,
            resources,
            record_updates,
        }
    }

    pub(in crate::asset::pipeline::manager) fn commit_targeted_project_resource_sync<T>(
        &self,
        prepared: PreparedTargetedProjectResourceSync,
        commit_files: impl FnOnce() -> Result<T, CoreError>,
        commit_project_state: impl FnOnce(),
    ) -> Result<T, CoreError> {
        let mut batch = ResourceMutationBatch::new();
        for locator in prepared.removed_locators {
            batch = batch.remove(locator);
        }
        for record in prepared.record_updates {
            batch = batch.upsert_lazy(record);
        }
        let batch = append_prepared_resources(batch, prepared.resources);
        self.commit_resource_batch_after_dependencies(batch, || {
            let outcome = commit_files()?;
            {
                let _phase = ProjectGenerationPhase::ProjectInstall.enter();
                self.project_source_paths_write()
                    .entry(prepared.source_uri.scheme())
                    .or_default()
                    .insert(prepared.source_uri.path().to_string(), prepared.source_path);
                commit_project_state();
            }
            Ok(outcome)
        })
    }

    pub(in crate::asset::pipeline::manager) fn commit_resource_batch_after_dependencies<T>(
        &self,
        batch: ResourceMutationBatch,
        commit_dependencies: impl FnOnce() -> Result<T, CoreError>,
    ) -> Result<T, CoreError> {
        let prepared_resource = {
            let _phase = ProjectGenerationPhase::ResourceReservation.enter();
            self.resource_manager
                .prepare_commit(batch)
                .map_err(asset_error)?
        };
        let outcome = commit_dependencies()?;
        {
            let _phase = ProjectGenerationPhase::ResourceApply.enter();
            prepared_resource.commit();
        }
        Ok(outcome)
    }
}
