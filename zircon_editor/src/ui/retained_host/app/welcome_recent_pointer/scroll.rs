use super::super::*;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn welcome_recent_pointer_scrolled(
        &mut self,
        x: f32,
        y: f32,
        delta: f32,
        width: f32,
        height: f32,
    ) {
        self.use_committed_pointer_layout();
        self.welcome_recent_pointer_size = self.resolve_callback_surface_size_for_kind(
            width,
            height,
            self.welcome_recent_pointer_size,
            ViewContentKind::Welcome,
        );
        let size_state_changed = self.sync_welcome_recent_pointer_size();
        let dispatch = self
            .welcome_recent_pointer_bridge
            .handle_scroll(UiPoint::new(x, y), delta);
        if size_state_changed || dispatch.changed {
            self.apply_welcome_recent_pointer_state_to_ui();
        }
    }
}
