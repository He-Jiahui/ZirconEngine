use super::super::*;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn console_pointer_scrolled(
        &mut self,
        x: f32,
        y: f32,
        delta: f32,
        width: f32,
        height: f32,
    ) {
        self.use_committed_pointer_layout();
        self.focus_callback_source_window();
        self.console_scroll_surface
            .set_size(self.resolve_callback_surface_size_for_kind(
                width,
                height,
                self.console_scroll_surface.size(),
                ViewContentKind::Console,
            ));
        let status_line = self.runtime.status_line();
        self.sync_console_pointer_layout(&status_line);
        match self
            .console_scroll_surface
            .handle_scroll(UiPoint::new(x, y), delta)
        {
            Ok(()) => {
                self.apply_console_pointer_state_to_ui();
            }
            Err(error) => self.set_status_line(error),
        }
    }
}
