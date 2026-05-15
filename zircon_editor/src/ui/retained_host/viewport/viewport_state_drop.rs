use super::viewport_state::ViewportState;

impl Drop for ViewportState {
    fn drop(&mut self) {
        if let (Some(viewport), Some(render_framework)) = (self.viewport, &self.render_framework) {
            let _ = render_framework.destroy_viewport(viewport.handle);
        }
    }
}
