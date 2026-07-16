use super::viewport_state::ViewportState;

impl Drop for ViewportState {
    fn drop(&mut self) {
        if let Some(cancel) = &self.render_framework_cancel {
            cancel.cancel();
        }
        if let (Some(jobs), Some(task)) = (&self.jobs, &self.render_framework_task) {
            jobs.cancel(task.id());
        }
        if let Some(viewport) = self.viewport {
            if let Ok(Some(render_framework)) = self.resolve_stored_render_framework() {
                let _ = render_framework.destroy_viewport(viewport.handle);
            }
        }
    }
}
