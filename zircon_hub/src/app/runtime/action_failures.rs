use std::path::PathBuf;

use crate::error::HubError;
use crate::state::{
    HubActionKind, HubActionRecord, HubActionStatus, TaskOperationKind, TaskStatus,
};

use super::HubRuntime;

impl HubRuntime {
    /// Persist a failed build action before returning from selection or validation gates.
    /// Build entry points use this for errors that happen before `PendingBuild` exists.
    pub(super) fn record_build_action_failure(
        &mut self,
        target: String,
        detail: String,
        command_line: Vec<String>,
        output_dir: Option<PathBuf>,
        recovery: &str,
    ) -> Result<(), HubError> {
        self.record_action_and_persist(HubActionRecord {
            finished_unix_ms: crate::projects::now_unix_ms(),
            action: HubActionKind::BuildEditorRuntime,
            status: HubActionStatus::Failed,
            target: target.clone(),
            detail: detail.clone(),
            log_excerpt: detail.clone(),
            recovery: Some(recovery.to_string()),
            process_id: None,
            command_line,
            output_dir,
        })?;
        self.task_status = TaskStatus::error("Build editor/runtime failed", detail, recovery)
            .with_operation(TaskOperationKind::Build, target);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::runtime::tests::{runtime_with_config_path, temp_test_dir};
    use crate::settings::HubConfig;

    #[test]
    fn early_build_failure_persists_recoverable_action_record() {
        let temp = temp_test_dir("zircon-hub-early-build-action-history");
        let config_path = temp.join("hub.toml");
        let mut runtime = runtime_with_config_path(Vec::new(), config_path.clone());
        runtime
            .record_build_action_failure(
                "Project".to_string(),
                "Select a project before building".to_string(),
                Vec::new(),
                Some(temp.join("out")),
                "Select a valid project with a bound Source Engine before building",
            )
            .unwrap();

        let reloaded = HubConfig::load(&config_path).unwrap();
        let record = &reloaded.action_history[0];
        assert_eq!(record.action, HubActionKind::BuildEditorRuntime);
        assert_eq!(record.status, HubActionStatus::Failed);
        assert_eq!(record.target, "Project");
        assert_eq!(record.log_excerpt, "Select a project before building");
        assert!(record
            .recovery
            .as_deref()
            .unwrap()
            .contains("bound Source Engine"));
        assert_eq!(runtime.task_status.label, "Build editor/runtime failed");
        assert_eq!(runtime.task_status.operation_summary(), "Build: Project");
        std::fs::remove_dir_all(temp).unwrap();
    }
}
