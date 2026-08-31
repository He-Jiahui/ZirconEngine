use super::super::*;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn welcome_recent_pointer_clicked(
        &mut self,
        x: f32,
        y: f32,
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
        if !self.ensure_welcome_surface_bridge() {
            if size_state_changed {
                self.apply_welcome_recent_pointer_state_to_ui();
            }
            return;
        }
        let Some(welcome_surface_bridge) = self.welcome_surface_bridge.as_ref() else {
            if size_state_changed {
                self.apply_welcome_recent_pointer_state_to_ui();
            }
            self.set_status_line("Welcome UI controls are not available");
            return;
        };
        match callback_dispatch::dispatch_shared_welcome_recent_pointer_click(
            welcome_surface_bridge,
            &mut self.welcome_recent_pointer_bridge,
            UiPoint::new(x, y),
        ) {
            Ok(dispatch) => {
                if size_state_changed || dispatch.pointer.changed {
                    self.apply_welcome_recent_pointer_state_to_ui();
                }
                if let Some(event) = dispatch.event {
                    self.handle_welcome_surface_event(event);
                }
            }
            Err(error) => {
                if size_state_changed {
                    self.apply_welcome_recent_pointer_state_to_ui();
                }
                self.set_status_line(error);
            }
        }
    }
}
