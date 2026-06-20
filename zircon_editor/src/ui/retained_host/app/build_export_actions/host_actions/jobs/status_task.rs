use crate::ui::retained_host::app::RetainedEditorHost;

use super::super::super::desktop_export_status_task_from_queue;

impl RetainedEditorHost {
    pub(super) fn sync_desktop_export_status_task(&mut self) {
        self.set_status_task_progress(desktop_export_status_task_from_queue(
            &self.desktop_export_jobs,
        ));
    }
}
