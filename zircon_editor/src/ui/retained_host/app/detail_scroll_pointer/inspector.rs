use super::super::*;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn inspector_pointer_scrolled(
        &mut self,
        x: f32,
        y: f32,
        delta: f32,
        width: f32,
        height: f32,
    ) {
        self.use_committed_pointer_layout();
        self.focus_callback_source_window();
        self.inspector_scroll_surface
            .set_size(self.resolve_callback_surface_size_for_kind(
                width,
                height,
                self.inspector_scroll_surface.size(),
                ViewContentKind::Inspector,
            ));
        self.sync_inspector_pointer_layout();
        match self
            .inspector_scroll_surface
            .handle_scroll(UiPoint::new(x, y), delta)
        {
            Ok(()) => {
                self.apply_inspector_pointer_state_to_ui();
            }
            Err(error) => self.set_status_line(error),
        }
    }
}
