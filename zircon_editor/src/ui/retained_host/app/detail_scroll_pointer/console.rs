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
        let size = self.resolve_callback_surface_size_for_kind(
            width,
            height,
            self.console_scroll_surface.size(),
            ViewContentKind::Console,
        );
        if self.console_scroll_surface.set_size(size) {
            let console_output = self.runtime.console_output();
            self.sync_console_pointer_layout(console_output.as_ref());
        }
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
