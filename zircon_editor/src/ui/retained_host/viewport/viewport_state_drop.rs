use super::viewport_state::ViewportState;

impl Drop for ViewportState {
    fn drop(&mut self) {
        if let Some(viewport) = self.viewport {
            let _ = self.render_framework.destroy_viewport(viewport.handle);
        }
    }
}
