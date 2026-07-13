use crate::core::jobs::EditorJobSystem;

use super::retained_viewport_controller::RetainedViewportController;

impl RetainedViewportController {
    pub(in crate::ui::retained_host) fn bind_jobs(&self, jobs: EditorJobSystem) {
        self.lock_shared().bind_jobs(jobs);
    }
}
