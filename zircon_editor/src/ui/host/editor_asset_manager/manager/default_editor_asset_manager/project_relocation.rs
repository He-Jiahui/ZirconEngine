use std::any::Any;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

use zircon_runtime::asset::pipeline::manager::ProjectAssetManager;
use zircon_runtime::asset::{AssetUri, AssetUuid};
use zircon_runtime::core::CoreError;

use crate::core::asset::{EditorAssetRelocationResult, EditorAssetRelocationTicket};
use crate::core::jobs::{
    EditorJob, EditorJobSpec, JobCategory, JobContext, JobError, JobPriority, MutexGroup,
};
use crate::ui::host::module::EDITOR_ASSET_MANAGER_NAME;

use super::DefaultEditorAssetManager;

const PROJECT_ASSET_MUTATION_MUTEX: &str = "project_asset_mutation";

struct ProjectSourceRelocationJob {
    manager: Arc<ProjectAssetManager>,
    source_uuid: AssetUuid,
    target: AssetUri,
}

impl EditorJob for ProjectSourceRelocationJob {
    type Output = EditorAssetRelocationResult;

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        context.check_cancelled()?;
        context.report_progress(0, 1, format!("Moving asset to {}", self.target));
        let result = match catch_unwind(AssertUnwindSafe(|| {
            self.manager
                .relocate_project_source(self.source_uuid.clone(), self.target.clone())
        })) {
            Ok(result) => result.map_err(JobError::failed),
            Err(payload) => Err(JobError::Panicked(panic_message(payload))),
        }?;
        context.report_progress(1, 1, format!("Moved asset to {}", self.target));
        Ok(EditorAssetRelocationResult::new(
            self.source_uuid,
            self.target,
            result,
        ))
    }
}

impl DefaultEditorAssetManager {
    /// Queues the Runtime-owned filesystem/registry/resource transaction outside the UI thread.
    pub fn submit_project_source_relocation(
        &self,
        source_uuid: AssetUuid,
        target: AssetUri,
    ) -> Result<EditorAssetRelocationTicket, CoreError> {
        let access = self.project_asset_manager.as_ref().ok_or_else(|| {
            CoreError::Initialization(
                EDITOR_ASSET_MANAGER_NAME.to_string(),
                "project source relocation requires the runtime asset manager".to_string(),
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
                    format!("Move asset to {target}"),
                    JobCategory::InteractiveSave,
                )
                .with_priority(JobPriority::Interactive)
                .with_mutex_group(mutex),
                ProjectSourceRelocationJob {
                    manager: asset_manager,
                    source_uuid,
                    target: target.clone(),
                },
            )
            .map_err(|error| CoreError::ServiceUnavailable(error.to_string()))?;
        Ok(EditorAssetRelocationTicket::new(ticket, target))
    }
}

fn panic_message(payload: Box<dyn Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string project asset relocation panic payload".to_owned()
    }
}

#[cfg(test)]
mod tests {
    use zircon_runtime::asset::{AssetUri, AssetUuid};
    use zircon_runtime::core::CoreError;

    use super::DefaultEditorAssetManager;

    #[test]
    fn relocation_requires_the_runtime_asset_owner() {
        let error = DefaultEditorAssetManager::new()
            .submit_project_source_relocation(
                AssetUuid::from_stable_label("editor-relocation-requires-runtime"),
                AssetUri::parse("res://relocated.asset").unwrap(),
            )
            .unwrap_err();

        assert_eq!(
            error,
            CoreError::Initialization(
                "EditorAssetManager".to_string(),
                "project source relocation requires the runtime asset manager".to_string(),
            )
        );
    }
}
