use crate::asset::project::ProjectGenerationPhase;
use crate::asset::watch::{AssetChange, AssetChangeKind};
use crate::asset::{AssetStatusRecord, AssetUuid};
use crate::core::CoreError;

use super::super::errors::{asset_error, asset_error_message};
use super::super::records::build_status_record;
use super::ProjectAssetManager;

impl ProjectAssetManager {
    /// Deletes an unreferenced authored source and its sidecar through one durable generation.
    pub fn delete_project_source(
        &self,
        target_uuid: AssetUuid,
    ) -> Result<Vec<AssetStatusRecord>, CoreError> {
        let (expected_generation, expected_preparation_epoch, mut candidate) = {
            let _generation = self.project_generation_read();
            let project = self.project_read();
            let Some(active_project) = project.as_ref() else {
                return Err(asset_error_message(
                    "project source deletion requires an active project",
                ));
            };
            (
                active_project.catalog_input_generation().sequence(),
                self.current_project_preparation_epoch(),
                active_project.clone(),
            )
        };
        let prepared_files = candidate
            .prepare_project_source_deletion(target_uuid)
            .map_err(asset_error)?;
        let statuses = prepared_files
            .removed_records()
            .iter()
            .map(build_status_record)
            .collect::<Vec<_>>();
        let source = prepared_files.source().clone();
        let prepared_resources =
            self.prepare_project_source_deletion_resource_sync(&prepared_files);

        let _phase = ProjectGenerationPhase::FileCommit.enter();
        let generation = self.project_generation_write();
        let mut project = self.project_write();
        let Some(active_project) = project.as_ref() else {
            return Err(asset_error_message(
                "project source deletion lost its active project before commit",
            ));
        };
        if active_project.catalog_input_generation().sequence() != expected_generation
            || self.current_project_preparation_epoch() != expected_preparation_epoch
        {
            return Err(asset_error_message(
                "project source deletion was superseded by a newer project generation",
            ));
        }
        let commit_outcome = self.commit_project_source_deletion_resource_sync(
            prepared_resources,
            || prepared_files.commit().map_err(asset_error),
            || {
                *project = Some(candidate);
                drop(project);
            },
        )?;
        self.publish_project_generation(
            generation,
            vec![AssetChange::new(AssetChangeKind::Removed, source, None)],
        );
        commit_outcome.ensure_durable().map_err(asset_error)?;
        Ok(statuses)
    }
}
