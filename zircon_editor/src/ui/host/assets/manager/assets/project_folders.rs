use std::any::Any;
use std::panic::{catch_unwind, AssertUnwindSafe};

use zircon_runtime::asset::project::ProjectActivationIdentity;
use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::CoreError;

use crate::core::asset::{
    EditorAssetFolderOperation, EditorAssetFolderResult, EditorAssetFolderTicket,
};
use crate::core::jobs::{
    EditorJob, EditorJobSpec, JobCategory, JobContext, JobError, JobPriority, MutexGroup,
};
use crate::ui::host::runtime_services::EditorProjectAssetRuntimeAccess;

use super::project_mutation_guard::mutation_error;
use super::DefaultEditorAssetManager;

const PROJECT_ASSET_MUTATION_MUTEX: &str = "project_asset_mutation";

struct ProjectFolderMutationJob {
    access: EditorProjectAssetRuntimeAccess,
    projection: DefaultEditorAssetManager,
    activation: ProjectActivationIdentity,
    operation: EditorAssetFolderOperation,
    source: AssetUri,
    target: Option<AssetUri>,
}

impl EditorJob for ProjectFolderMutationJob {
    type Output = EditorAssetFolderResult;

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        context.check_cancelled()?;
        context.report_progress(0, 1, format!("Updating folder {}", self.source));
        let statuses = match catch_unwind(AssertUnwindSafe(|| {
            self.projection.require_clean_folder_mutation(
                &self.activation,
                self.operation,
                &self.source,
            )?;
            let manager = self.access.project_asset_manager()?;
            let statuses = match self.operation {
                EditorAssetFolderOperation::Create => {
                    manager
                        .create_project_folder_for_project(&self.activation, self.source.clone())?;
                    Vec::new()
                }
                EditorAssetFolderOperation::Rename => {
                    let target = self
                        .target
                        .clone()
                        .ok_or_else(|| mutation_error("folder rename requires a destination"))?;
                    manager.relocate_project_folder_for_project(
                        &self.activation,
                        self.source.clone(),
                        target,
                    )?
                }
                EditorAssetFolderOperation::Delete => {
                    manager
                        .delete_project_folder_for_project(&self.activation, self.source.clone())?;
                    Vec::new()
                }
            };
            drop(manager);
            self.projection.refresh_from_runtime_project()?;
            Ok::<_, CoreError>(statuses)
        })) {
            Ok(result) => result.map_err(JobError::failed),
            Err(payload) => Err(JobError::Panicked(panic_message(payload))),
        }?;
        context.report_progress(1, 1, format!("Updated folder {}", self.source));
        Ok(EditorAssetFolderResult::new(
            self.operation,
            self.source,
            self.target,
            statuses,
        ))
    }
}

impl DefaultEditorAssetManager {
    pub fn submit_project_folder_mutation(
        &self,
        operation: EditorAssetFolderOperation,
        source: AssetUri,
        target: Option<AssetUri>,
    ) -> Result<EditorAssetFolderTicket, CoreError> {
        let access = self.project_asset_manager.as_ref().ok_or_else(|| {
            mutation_error("project folder mutation requires the runtime asset manager")
        })?;
        if (operation == EditorAssetFolderOperation::Rename) != target.is_some() {
            return Err(mutation_error(
                "only a folder rename accepts and requires a destination",
            ));
        }
        let activation = self.project_mutation_activation()?;
        self.require_clean_folder_mutation(&activation, operation, &source)?;
        if !access
            .project_asset_manager()?
            .is_project_activation_current(&activation)
        {
            return Err(mutation_error(
                "folder operation belongs to an inactive runtime project",
            ));
        }
        let jobs = self
            .preview_jobs
            .as_ref()
            .ok_or_else(|| CoreError::ServiceUnavailable("EditorJobSystem".to_owned()))?;
        let mutex = MutexGroup::parse(PROJECT_ASSET_MUTATION_MUTEX)
            .map_err(|error| mutation_error(error.to_string()))?;
        let ticket = jobs
            .submit(
                EditorJobSpec::new(
                    format!("{operation:?} folder {source}"),
                    JobCategory::InteractiveSave,
                )
                .with_priority(JobPriority::Interactive)
                .with_mutex_group(mutex),
                ProjectFolderMutationJob {
                    access: access.clone(),
                    projection: self.clone(),
                    activation,
                    operation,
                    source: source.clone(),
                    target: target.clone(),
                },
            )
            .map_err(|error| CoreError::ServiceUnavailable(error.to_string()))?;
        Ok(EditorAssetFolderTicket::new(
            ticket, operation, source, target,
        ))
    }
}

fn panic_message(payload: Box<dyn Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string project folder mutation panic payload".to_owned()
    }
}

#[cfg(test)]
#[path = "tests/project_folders.rs"]
mod tests;
