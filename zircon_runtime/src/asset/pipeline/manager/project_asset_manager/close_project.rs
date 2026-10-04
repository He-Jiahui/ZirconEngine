use std::path::PathBuf;

use crate::asset::watch::{AssetChange, AssetChangeKind};
use crate::core::resource::ResourceMutationBatch;
use crate::core::CoreError;

use super::super::resource_sync::project_locators;
use super::ProjectAssetManager;

impl ProjectAssetManager {
    /// Retires the active project generation and returns its committed root.
    ///
    /// The generation write lock prevents watcher callbacks from observing a partially retired
    /// project while the watcher activation, resource registry, source-path index, and project
    /// snapshot transition together. It also spans `Removed` publication so a later generation's
    /// `Added` publication cannot overtake the close event.
    pub(crate) fn close_project(&self) -> Result<Option<PathBuf>, CoreError> {
        let _watcher_operation = self.begin_project_watcher_operation()?;
        let generation = self.project_generation_write();
        let (root, removed_changes, retired_watchers) = {
            let mut project = self.project_write();
            let Some(active_project) = project.as_ref() else {
                return Ok(None);
            };
            self.begin_project_preparation();
            let root = active_project.paths().root().to_path_buf();
            let locators = project_locators(active_project);
            let removed_changes = locators
                .iter()
                .cloned()
                .map(|uri| AssetChange::new(AssetChangeKind::Removed, uri, None))
                .collect();

            let mut batch = ResourceMutationBatch::new();
            for locator in locators {
                batch = batch.remove(locator);
            }
            let mut retired_watchers = None;
            self.commit_resource_batch_after_dependencies(batch, || {
                retired_watchers = Some(self.deactivate_project_watchers());
                self.clear_project_source_paths();
                self.clear_transaction_watch_echoes();
                *project = None;
                drop(project);
                Ok(())
            })?;

            (
                root,
                removed_changes,
                retired_watchers.expect("successful project retirement deactivates its watchers"),
            )
        };

        self.publish_project_generation(generation, removed_changes);
        drop(retired_watchers);
        Ok(Some(root))
    }
}

#[cfg(test)]
#[path = "tests/close_project.rs"]
mod tests;
