//! Runtime owns folder filesystem changes, activation fences and catalog change publication.

use crate::asset::project::{ProjectActivationIdentity, ProjectGenerationPhase, ProjectManager};
use crate::asset::watch::{AssetChange, AssetChangeKind};
use crate::asset::{AssetStatusRecord, AssetUri};
use crate::core::CoreError;

use super::super::errors::{asset_error, asset_error_message};
use super::super::records::build_status_record;
use super::ProjectAssetManager;

impl ProjectAssetManager {
    pub fn create_project_folder(&self, folder: AssetUri) -> Result<(), CoreError> {
        self.change_project_folder(None, folder, true)
    }

    /// Queued editor work must remain bound to the activation that admitted the request.
    pub fn create_project_folder_for_project(
        &self,
        expected: &ProjectActivationIdentity,
        folder: AssetUri,
    ) -> Result<(), CoreError> {
        self.change_project_folder(Some(expected), folder, true)
    }

    pub fn delete_project_folder(&self, folder: AssetUri) -> Result<(), CoreError> {
        self.change_project_folder(None, folder, false)
    }

    pub fn delete_project_folder_for_project(
        &self,
        expected: &ProjectActivationIdentity,
        folder: AssetUri,
    ) -> Result<(), CoreError> {
        self.change_project_folder(Some(expected), folder, false)
    }

    pub fn relocate_project_folder(
        &self,
        source: AssetUri,
        target: AssetUri,
    ) -> Result<Vec<AssetStatusRecord>, CoreError> {
        self.relocate_project_folder_for_scope(None, source, target)
    }

    pub fn relocate_project_folder_for_project(
        &self,
        expected: &ProjectActivationIdentity,
        source: AssetUri,
        target: AssetUri,
    ) -> Result<Vec<AssetStatusRecord>, CoreError> {
        self.relocate_project_folder_for_scope(Some(expected), source, target)
    }

    fn change_project_folder(
        &self,
        expected: Option<&ProjectActivationIdentity>,
        folder: AssetUri,
        create: bool,
    ) -> Result<(), CoreError> {
        let (expected_generation, expected_preparation_epoch, mut candidate) =
            self.project_for_folder_mutation(expected)?;
        let project_generation =
            crate::asset::project::lock_project_generation(candidate.paths().root())
                .map_err(asset_error)?;
        let _phase = ProjectGenerationPhase::FileCommit.enter();
        let generation = self.project_generation_write();
        let mut project = self.project_write();
        self.verify_folder_mutation_generation(
            project.as_ref(),
            &candidate,
            expected_generation,
            expected_preparation_epoch,
        )?;
        let commit_outcome = if create {
            candidate
                .create_project_folder(&folder)
                .map_err(asset_error)?
        } else {
            candidate
                .delete_project_folder(&folder)
                .map_err(asset_error)?
        };
        *project = Some(candidate);
        drop(project);
        drop(project_generation);
        self.publish_project_generation(
            generation,
            vec![AssetChange::new(
                if create {
                    AssetChangeKind::Added
                } else {
                    AssetChangeKind::Removed
                },
                folder,
                None,
            )],
        );
        commit_outcome.ensure_durable().map_err(asset_error)
    }

    fn relocate_project_folder_for_scope(
        &self,
        expected: Option<&ProjectActivationIdentity>,
        source: AssetUri,
        target: AssetUri,
    ) -> Result<Vec<AssetStatusRecord>, CoreError> {
        let (expected_generation, expected_preparation_epoch, mut candidate) =
            self.project_for_folder_mutation(expected)?;
        let project_generation =
            crate::asset::project::lock_project_generation(candidate.paths().root())
                .map_err(asset_error)?;
        let prepared_files = candidate
            .prepare_project_folder_relocation(source.clone(), target.clone())
            .map_err(asset_error)?;
        if prepared_files.no_change() {
            return Ok(Vec::new());
        }
        let statuses = prepared_files
            .relocation()
            .updated_records()
            .iter()
            .map(build_status_record)
            .collect::<Vec<_>>();
        let mut changes = prepared_files
            .relocation()
            .source_path_moves()
            .iter()
            .map(|(previous, current, _)| {
                AssetChange::new(
                    AssetChangeKind::Renamed,
                    current.clone(),
                    Some(previous.clone()),
                )
            })
            .collect::<Vec<_>>();
        changes.push(AssetChange::new(
            AssetChangeKind::Renamed,
            target,
            Some(source),
        ));
        let prepared_resources =
            self.prepare_project_source_relocation_resource_sync(prepared_files.relocation());
        let _phase = ProjectGenerationPhase::FileCommit.enter();
        let generation = self.project_generation_write();
        let mut project = self.project_write();
        self.verify_folder_mutation_generation(
            project.as_ref(),
            &candidate,
            expected_generation,
            expected_preparation_epoch,
        )?;
        let commit_outcome = self.commit_project_source_relocation_resource_sync(
            prepared_resources,
            || prepared_files.commit().map_err(asset_error),
            || {
                *project = Some(candidate);
                drop(project);
            },
        )?;
        drop(project_generation);
        self.publish_project_generation(generation, changes);
        commit_outcome.ensure_durable().map_err(asset_error)?;
        Ok(statuses)
    }

    fn project_for_folder_mutation(
        &self,
        expected: Option<&ProjectActivationIdentity>,
    ) -> Result<(u64, u64, ProjectManager), CoreError> {
        let _generation = self.project_generation_read();
        let project = self.project_read();
        let active = project.as_ref().ok_or_else(|| {
            asset_error_message("project folder mutation requires an active project")
        })?;
        if expected.is_some_and(|expected| active.activation_identity() != expected) {
            return Err(asset_error_message(
                "project folder mutation activation was superseded",
            ));
        }
        Ok((
            active.catalog_input_generation().sequence(),
            self.current_project_preparation_epoch(),
            active.clone(),
        ))
    }

    fn verify_folder_mutation_generation(
        &self,
        active: Option<&ProjectManager>,
        candidate: &ProjectManager,
        expected_generation: u64,
        expected_preparation_epoch: u64,
    ) -> Result<(), CoreError> {
        if active.is_none_or(|active| {
            active.activation_identity() != candidate.activation_identity()
                || active.catalog_input_generation().sequence() != expected_generation
                || self.current_project_preparation_epoch() != expected_preparation_epoch
        }) {
            return Err(asset_error_message(
                "project folder mutation was superseded by a newer project generation",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/folders.rs"]
mod tests;
