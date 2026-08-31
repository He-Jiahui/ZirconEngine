use crate::ui::retained_host::app::{HostInvalidationMask, RetainedEditorHost};
use crate::ui::template_runtime::WORKBENCH_WINDOW_DOCUMENT_ID;
use crate::ui::workbench::snapshot::StatusTaskProgressSnapshot;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn set_status_line(
        &mut self,
        message: impl Into<String>,
    ) {
        let message = message.into();
        if !self.runtime.set_retained_status_line(&message) {
            return;
        }
        if !self.active_activity_window_template_document_is(WORKBENCH_WINDOW_DOCUMENT_ID) {
            self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
            return;
        }
        let patched = self
            .workbench_window_bridge
            .prepare_status_line(&message)
            .and_then(|()| self.workbench_window_bridge.refresh_prepared_state_change())
            .is_ok();
        self.invalidate_host(if patched {
            HostInvalidationMask::WORKBENCH_PROJECTION
        } else {
            HostInvalidationMask::PRESENTATION_DATA
        });
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
