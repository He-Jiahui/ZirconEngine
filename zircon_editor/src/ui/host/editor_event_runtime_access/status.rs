use crate::core::editor_message::EditorViewInvalidationMask;
use crate::core::jobs::EditorJobProgressSnapshot;
use crate::ui::host::editor_activity_log::activity_log_console_output;
use crate::ui::host::EditorHostEventController;
use crate::ui::workbench::snapshot::{ConsoleOutputSnapshot, StatusTaskProgressSnapshot};

impl EditorHostEventController {
    pub fn set_status_line(&self, message: impl Into<String>) {
        self.shell().lock().state.set_status_line(message);
        self.refresh_workbench(EditorViewInvalidationMask::PRESENTATION_DATA);
    }

    pub fn status_line(&self) -> String {
        self.shell().lock().state.status_line.clone()
    }

    pub(crate) fn set_retained_status_line(&self, message: impl Into<String>) -> bool {
        let message = message.into();
        let mut shell = self.shell().lock();
        if shell.state.status_line == message {
            return false;
        }
        shell.state.set_status_line(message);
        true
    }

    pub(crate) fn console_output(&self) -> ConsoleOutputSnapshot {
        let shell = self.shell().lock();
        activity_log_console_output(
            shell.manager.context().logs(),
            shell.console_message_filter,
            shell.console_source_filter,
        )
    }

    pub fn set_status_task_progress(&self, progress: Option<StatusTaskProgressSnapshot>) {
        self.shell().lock().state.set_status_task_progress(progress);
        self.refresh_workbench(EditorViewInvalidationMask::PRESENTATION_DATA);
    }

    pub(crate) fn set_retained_status_task_progress(
        &self,
        progress: Option<StatusTaskProgressSnapshot>,
    ) -> bool {
        let mut shell = self.shell().lock();
        if shell.state.status_task_progress == progress {
            return false;
        }
        shell.state.set_status_task_progress(progress);
        true
    }

    pub fn status_task_progress(&self) -> Option<StatusTaskProgressSnapshot> {
        self.shell().lock().state.status_task_progress.clone()
    }

    pub fn job_progress_snapshot(&self) -> Vec<EditorJobProgressSnapshot> {
        self.context().jobs().progress().snapshot()
    }

    pub fn primary_job_progress_snapshot(&self) -> Option<EditorJobProgressSnapshot> {
        self.context().jobs().progress().primary_snapshot()
    }
}
