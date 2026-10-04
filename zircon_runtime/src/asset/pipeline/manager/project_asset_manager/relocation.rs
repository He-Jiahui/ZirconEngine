use crate::asset::project::ProjectGenerationPhase;
use crate::asset::watch::{AssetChange, AssetChangeKind};
use crate::asset::{AssetStatusRecord, AssetUri, AssetUuid};
use crate::core::CoreError;

use super::super::errors::{asset_error, asset_error_message};
use super::super::records::build_status_record;
use super::ProjectAssetManager;

impl ProjectAssetManager {
    /// Relocates an active project source through the Runtime asset pipeline.
    ///
    /// This is the only public mutation entry: it prepares a project candidate, reserves the
    /// live resource rename batch, commits authoring files, then installs the project generation
    /// and publishes the change while the generation gate is held.
    pub fn relocate_project_source(
        &self,
        source_uuid: AssetUuid,
        target: AssetUri,
    ) -> Result<Vec<AssetStatusRecord>, CoreError> {
        let (expected_generation, expected_preparation_epoch, mut candidate) = {
            let _generation = self.project_generation_read();
            let project = self.project_read();
            let Some(active_project) = project.as_ref() else {
                return Err(asset_error_message(
                    "project source relocation requires an active project",
                ));
            };
            (
                active_project.catalog_input_generation().sequence(),
                self.current_project_preparation_epoch(),
                active_project.clone(),
            )
        };
        let project_generation =
            crate::asset::project::lock_project_generation(candidate.paths().root())
                .map_err(asset_error)?;
        let prepared_files = candidate
            .prepare_project_source_relocation(source_uuid, target)
            .map_err(asset_error)?;
        if prepared_files.updated_records().is_empty() {
            return Ok(Vec::new());
        }
        let statuses = prepared_files
            .updated_records()
            .iter()
            .map(build_status_record)
            .collect::<Vec<_>>();
        let source = prepared_files.source().clone();
        let target = prepared_files.target().clone();
        let prepared_resources =
            self.prepare_project_source_relocation_resource_sync(&prepared_files);

        let _phase = ProjectGenerationPhase::FileCommit.enter();
        let generation = self.project_generation_write();
        let mut project = self.project_write();
        let Some(active_project) = project.as_ref() else {
            return Err(asset_error_message(
                "project source relocation lost its active project before commit",
            ));
        };
        if active_project.catalog_input_generation().sequence() != expected_generation
            || self.current_project_preparation_epoch() != expected_preparation_epoch
        {
            return Err(asset_error_message(
                "project source relocation was superseded by a newer project generation",
            ));
        }
        let commit_outcome = self.commit_project_source_relocation_resource_sync(
            prepared_resources,
            || prepared_files.commit().map_err(asset_error),
            || {
                *project = Some(candidate);
                drop(project);
            },
        )?;
        drop(project_generation);
        self.publish_project_generation(
            generation,
            vec![AssetChange::new(
                AssetChangeKind::Renamed,
                target,
                Some(source),
            )],
        );
        commit_outcome.ensure_durable().map_err(asset_error)?;
        Ok(statuses)
    }
}

#[cfg(test)]
#[path = "tests/relocation.rs"]
mod tests;
