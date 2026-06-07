#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TaskStatus {
    pub label: String,
    pub detail: String,
    pub running: bool,
    pub severity: TaskSeverity,
    pub recovery: Option<String>,
    pub operation: Option<TaskOperationKind>,
    pub target: Option<String>,
    pub progress_percent: u8,
}

pub const TASK_PROGRESS_IDLE_PERCENT: u8 = 0;
pub const TASK_PROGRESS_STARTED_PERCENT: u8 = 10;
pub const TASK_PROGRESS_PREPARED_PERCENT: u8 = 35;
pub const TASK_PROGRESS_COMPLETE_PERCENT: u8 = 100;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TaskSeverity {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskOperationKind {
    Project,
    SourceEngine,
    Build,
    Process,
    Settings,
    Hub,
}

impl TaskStatus {
    pub fn idle() -> Self {
        Self {
            label: "Ready".to_string(),
            detail: "Hub is ready".to_string(),
            running: false,
            severity: TaskSeverity::Info,
            recovery: None,
            operation: Some(TaskOperationKind::Hub),
            target: None,
            progress_percent: TASK_PROGRESS_IDLE_PERCENT,
        }
    }

    pub fn running(label: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            detail: detail.into(),
            running: true,
            severity: TaskSeverity::Info,
            recovery: None,
            operation: None,
            target: None,
            progress_percent: TASK_PROGRESS_STARTED_PERCENT,
        }
    }

    pub fn running_operation(
        label: impl Into<String>,
        detail: impl Into<String>,
        operation: TaskOperationKind,
        target: impl Into<String>,
    ) -> Self {
        Self::running(label, detail).with_operation(operation, target)
    }

    pub fn success(label: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::new(label, detail, TaskSeverity::Success, None)
    }

    pub fn warning(
        label: impl Into<String>,
        detail: impl Into<String>,
        recovery: impl Into<String>,
    ) -> Self {
        Self::new(label, detail, TaskSeverity::Warning, Some(recovery.into()))
    }

    pub fn error(
        label: impl Into<String>,
        detail: impl Into<String>,
        recovery: impl Into<String>,
    ) -> Self {
        Self::new(label, detail, TaskSeverity::Error, Some(recovery.into()))
    }

    fn new(
        label: impl Into<String>,
        detail: impl Into<String>,
        severity: TaskSeverity,
        recovery: Option<String>,
    ) -> Self {
        Self {
            label: label.into(),
            detail: detail.into(),
            running: false,
            severity,
            recovery,
            operation: None,
            target: None,
            progress_percent: match severity {
                TaskSeverity::Success => TASK_PROGRESS_COMPLETE_PERCENT,
                _ => TASK_PROGRESS_IDLE_PERCENT,
            },
        }
    }

    pub fn with_progress_percent(mut self, progress_percent: u8) -> Self {
        self.progress_percent = progress_percent.min(TASK_PROGRESS_COMPLETE_PERCENT);
        self
    }

    pub fn set_progress_percent(&mut self, progress_percent: u8) {
        self.progress_percent = progress_percent.min(TASK_PROGRESS_COMPLETE_PERCENT);
    }

    pub fn with_operation(
        mut self,
        operation: TaskOperationKind,
        target: impl Into<String>,
    ) -> Self {
        let target = target.into();
        self.operation = Some(operation);
        self.target = (!target.trim().is_empty()).then_some(target);
        self
    }

    pub fn operation_summary(&self) -> String {
        let Some(operation) = self.operation else {
            return self.label.clone();
        };
        let scope = match operation {
            TaskOperationKind::Project => "Project",
            TaskOperationKind::SourceEngine => "Source Engine",
            TaskOperationKind::Build => "Build",
            TaskOperationKind::Process => "Process",
            TaskOperationKind::Settings => "Settings",
            TaskOperationKind::Hub => "Hub",
        };
        match self
            .target
            .as_deref()
            .filter(|target| !target.trim().is_empty())
        {
            Some(target) => format!("{scope}: {target}"),
            None => scope.to_string(),
        }
    }

    pub fn detail_with_recovery(&self) -> String {
        match self
            .recovery
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            Some(recovery) if self.detail.trim().is_empty() => recovery.to_string(),
            Some(recovery) => format!("{} — {}", self.detail, recovery),
            None => self.detail.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_status_preserves_recovery_without_hiding_detail() {
        let status = TaskStatus::error(
            "Build failed",
            "cargo exited with code 101",
            "Review build history and fix the first compiler error",
        );

        assert_eq!(status.severity, TaskSeverity::Error);
        assert!(!status.running);
        assert_eq!(
            status.detail_with_recovery(),
            "cargo exited with code 101 — Review build history and fix the first compiler error"
        );
    }

    #[test]
    fn task_status_operation_summary_names_scope_and_target() {
        let status = TaskStatus::success("Project selected", "Game")
            .with_operation(TaskOperationKind::Project, "Game");

        assert_eq!(status.operation_summary(), "Project: Game");
    }

    #[test]
    fn task_status_progress_tracks_lifecycle_checkpoints() {
        assert_eq!(
            TaskStatus::idle().progress_percent,
            TASK_PROGRESS_IDLE_PERCENT
        );
        assert_eq!(
            TaskStatus::running("Building", "Running tools/zircon_build.py").progress_percent,
            TASK_PROGRESS_STARTED_PERCENT
        );
        assert_eq!(
            TaskStatus::success("Build complete", "out").progress_percent,
            TASK_PROGRESS_COMPLETE_PERCENT
        );
        assert_eq!(
            TaskStatus::error("Build failed", "failed", "retry").progress_percent,
            TASK_PROGRESS_IDLE_PERCENT
        );

        let clamped = TaskStatus::running("Building", "Running tools/zircon_build.py")
            .with_progress_percent(TASK_PROGRESS_COMPLETE_PERCENT + 1);

        assert_eq!(clamped.progress_percent, TASK_PROGRESS_COMPLETE_PERCENT);
    }
}
