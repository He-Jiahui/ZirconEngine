use std::path::PathBuf;

use crate::engines::{active_source_engine, SourceEngineInstall};
use crate::error::HubError;
use crate::state::{HubActionKind, HubActionRecord, HubMessage, TaskOperationKind, TaskStatus};

use super::HubRuntimeSession;

impl HubRuntimeSession {
    pub(super) fn record_action_and_persist(
        &mut self,
        record: HubActionRecord,
    ) -> Result<(), HubError> {
        self.config.action_history.insert(0, record);
        self.config
            .action_history
            .truncate(crate::state::ACTION_HISTORY_LIMIT);
        self.persist()
    }

    pub(super) fn set_action_failure_status(
        &mut self,
        action: HubActionKind,
        target: String,
        detail: HubMessage,
        recovery: HubMessage,
    ) {
        self.task_status =
            TaskStatus::error(format!("{} failed", action.label()), detail, recovery)
                .with_operation(action_operation_kind(action), target);
    }

    pub(super) fn action_target_for_project_failure(&self) -> String {
        self.selected_project_path
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Project".to_string())
    }

    pub(super) fn staged_engine_dir(&self) -> PathBuf {
        active_source_engine(
            &self.config.engines,
            self.config.active_engine_id.as_deref(),
        )
        .map(SourceEngineInstall::staged_engine_dir)
        .unwrap_or_else(|| {
            self.config
                .settings
                .default_build_output_dir
                .join("ZirconEngine")
        })
    }
}

fn action_operation_kind(action: HubActionKind) -> TaskOperationKind {
    match action {
        HubActionKind::BuildEditorRuntime => TaskOperationKind::Build,
        HubActionKind::OpenEditor | HubActionKind::OpenOutput => TaskOperationKind::Process,
        HubActionKind::OpenResource => TaskOperationKind::Hub,
        HubActionKind::CreateProject
        | HubActionKind::ImportProject
        | HubActionKind::RemoveProject
        | HubActionKind::DeleteProject
        | HubActionKind::PackageProject
        | HubActionKind::InstallProject => TaskOperationKind::Project,
    }
}

#[cfg(test)]
#[path = "tests/quick_actions.rs"]
mod tests;
