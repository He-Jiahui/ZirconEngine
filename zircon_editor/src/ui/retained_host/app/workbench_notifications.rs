use super::*;
use crate::ui::retained_host::workbench_notifications::WorkbenchNotification;
use crate::ui::template_runtime::WORKBENCH_WINDOW_DOCUMENT_ID;

impl RetainedEditorHost {
    pub(super) fn publish_workbench_notifications(
        &mut self,
        notifications: &[WorkbenchNotification],
    ) {
        if notifications.is_empty()
            || !self.active_activity_window_template_document_is(WORKBENCH_WINDOW_DOCUMENT_ID)
        {
            return;
        }

        match self
            .workbench_window_bridge
            .push_workbench_notifications(notifications)
        {
            Ok(true) => self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA),
            Ok(false) => {}
            Err(error) => self.set_status_line(error.to_string()),
        }
    }
}
