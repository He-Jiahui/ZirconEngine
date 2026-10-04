use crate::asset::project::{lock_project_generation, ProjectManager};
use crate::asset::watch::{AssetChange, AssetChangeKind};
use crate::asset::ProjectInfo;
use crate::core::resource::ResourceMutationBatch;
use crate::core::CoreError;

use super::super::errors::{asset_error, asset_error_message};
use super::super::records::build_project_info;
use super::super::resource_sync::{clear_removed_project_resources, project_locators};
use super::ProjectAssetManager;

impl ProjectAssetManager {
    /// Activates an already parsed project as the authoritative runtime asset project.
    ///
    /// Runtime session construction uses this entry after reading the manifest once for plugin
    /// selection. The path-based [`crate::asset::AssetManager::open_project`] entry delegates here
    /// as well so both startup routes share importer registration, scanning, resource sync, watcher
    /// replacement, and change publication semantics.
    pub(crate) fn open_prepared_project(
        &self,
        mut project: ProjectManager,
    ) -> Result<ProjectInfo, CoreError> {
        // This scope also retains ownership of prepared and retired watcher threads until joined.
        let _watcher_operation = self.begin_project_watcher_operation()?;
        let preparation_epoch = self.begin_project_preparation();
        self.clear_transaction_watch_echoes();
        let installed_importers = self.importer_registry_read().clone();
        *project.importer_mut().registry_mut() = installed_importers;
        project.set_environment_ibl_parallel_executor(self.worker_task_pool.clone());
        let prepared_watchers = self.prepare_project_watchers(&project)?;
        let project_generation =
            lock_project_generation(project.paths().root()).map_err(asset_error)?;
        let (imported, prepared_files) =
            project.prepare_full_generation(None).map_err(asset_error)?;
        let prepared_resources = self.prepare_project_resource_sync(&project)?;
        let info = build_project_info(&project);
        let generation = self.project_generation_write();
        if !self.is_latest_project_preparation(preparation_epoch) {
            return Err(asset_error_message(
                "project activation was superseded by a newer preparation",
            ));
        }
        let watcher_publication = self.admit_project_watcher_publication()?;
        let previous_locators = self
            .project_read()
            .as_ref()
            .map(project_locators)
            .unwrap_or_default();
        let mut watcher_transition = None;
        let commit_outcome = {
            let mut active_project = self.project_write();
            let batch = clear_removed_project_resources(
                ResourceMutationBatch::new(),
                &previous_locators,
                &project,
            );
            self.commit_project_resource_sync(
                prepared_resources,
                batch,
                || prepared_files.commit().map_err(asset_error),
                || {
                    *active_project = Some(project);
                    watcher_transition = Some(
                        self.activate_project_watchers(prepared_watchers, &watcher_publication),
                    );
                    drop(active_project);
                },
            )?
        };
        let (retired_watchers, watcher_activation) = watcher_transition
            .expect("successful project publication installs its prepared watchers");
        drop(project_generation);
        drop(watcher_publication);
        self.publish_project_generation(
            generation,
            imported
                .into_iter()
                .map(|metadata| {
                    AssetChange::new(
                        AssetChangeKind::Added,
                        metadata.primary_locator().clone(),
                        None,
                    )
                })
                .collect(),
        );
        drop(retired_watchers);
        self.drain_project_watcher_events(watcher_activation);
        commit_outcome.ensure_durable().map_err(asset_error)?;
        Ok(info)
    }
}

#[cfg(test)]
#[path = "tests/open_project.rs"]
mod tests;
