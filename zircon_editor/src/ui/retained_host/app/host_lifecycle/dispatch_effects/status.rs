use crate::ui::retained_host::app::{HostInvalidationMask, RetainedEditorHost};
use crate::ui::workbench::snapshot::StatusTaskProgressSnapshot;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn set_status_line(
        &mut self,
        message: impl Into<String>,
    ) {
        let message = message.into();
        if self.runtime.status_line() == message {
            return;
        }
        self.runtime.set_status_line(message);
        self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
    }

    pub(in crate::ui::retained_host::app) fn set_status_task_progress(
        &mut self,
        progress: Option<StatusTaskProgressSnapshot>,
    ) {
        if self.runtime.status_task_progress() == progress {
            return;
        }
        self.runtime.set_status_task_progress(progress);
        self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
    }
}
