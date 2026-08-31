use std::any::Any;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

use zircon_runtime::asset::pipeline::manager::ProjectAssetManager;
use zircon_runtime::asset::AssetUuid;
use zircon_runtime::core::CoreError;

use crate::core::asset::{EditorAssetDeletionResult, EditorAssetDeletionTicket};
use crate::core::jobs::{
    EditorJob, EditorJobSpec, JobCategory, JobContext, JobError, JobPriority, MutexGroup,
};
use crate::ui::host::module::EDITOR_ASSET_MANAGER_NAME;

use super::DefaultEditorAssetManager;

const PROJECT_ASSET_MUTATION_MUTEX: &str = "project_asset_mutation";

struct ProjectSourceDeletionJob {
    manager: Arc<ProjectAssetManager>,
    target_uuid: AssetUuid,
}

impl EditorJob for ProjectSourceDeletionJob {
    type Output = EditorAssetDeletionResult;

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        context.check_cancelled()?;
        context.report_progress(0, 1, format!("Deleting asset {}", self.target_uuid));
        let result = match catch_unwind(AssertUnwindSafe(|| {
            self.manager.delete_project_source(self.target_uuid.clone())
        })) {
            Ok(result) => result.map_err(JobError::failed),
            Err(payload) => Err(JobError::Panicked(panic_message(payload))),
        }?;
        context.report_progress(1, 1, format!("Deleted asset {}", self.target_uuid));
        Ok(EditorAssetDeletionResult::new(self.target_uuid, result))
    }
}

impl DefaultEditorAssetManager {
    pub fn submit_project_source_deletion(
        &self,
        target_uuid: AssetUuid,
    ) -> Result<EditorAssetDeletionTicket, CoreError> {
        let access = self.project_asset_manager.as_ref().ok_or_else(|| {
            CoreError::Initialization(
                EDITOR_ASSET_MANAGER_NAME.to_string(),
                "project source deletion requires the runtime asset manager".to_string(),
            )
        })?;
        let asset_manager = access.project_asset_manager()?;
        let jobs = self
            .preview_jobs
            .as_ref()
            .cloned()
            .ok_or_else(|| CoreError::ServiceUnavailable("EditorJobSystem".to_owned()))?;
        let mutex = MutexGroup::parse(PROJECT_ASSET_MUTATION_MUTEX).map_err(|error| {
            CoreError::Initialization(EDITOR_ASSET_MANAGER_NAME.to_owned(), error.to_string())
        })?;
        let ticket = jobs
            .submit(
                EditorJobSpec::new(
                    format!("Delete asset {target_uuid}"),
                    JobCategory::InteractiveSave,
                )
                .with_priority(JobPriority::Interactive)
                .with_mutex_group(mutex),
                ProjectSourceDeletionJob {
                    manager: asset_manager,
                    target_uuid: target_uuid.clone(),
                },
            )
            .map_err(|error| CoreError::ServiceUnavailable(error.to_string()))?;
        Ok(EditorAssetDeletionTicket::new(ticket, target_uuid))
    }
}

fn panic_message(payload: Box<dyn Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string project asset deletion panic payload".to_owned()
    }
}
