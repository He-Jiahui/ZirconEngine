use super::*;

impl SlintEditorHost {
    pub(super) fn welcome_recent_pointer_clicked(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        self.welcome_recent_pointer_size = self.resolve_callback_surface_size_for_kind(
            width,
            height,
            self.welcome_recent_pointer_size,
            ViewContentKind::Welcome,
        );
        let welcome = self.runtime.chrome_snapshot().welcome;
        self.sync_welcome_recent_pointer_layout(&welcome);
        match callback_dispatch::dispatch_shared_welcome_recent_pointer_click(
            &self.welcome_surface_bridge,
            &mut self.welcome_recent_pointer_bridge,
            UiPoint::new(x, y),
        ) {
            Ok(dispatch) => {
                self.welcome_recent_pointer_state = dispatch.pointer.state;
                self.apply_welcome_recent_pointer_state_to_ui();
                if let Some(event) = dispatch.event {
                    self.handle_welcome_surface_event(event);
                }
            }
            Err(error) => self.set_status_line(error),
        }
    }

    pub(super) fn welcome_recent_pointer_moved(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.welcome_recent_pointer_size = self.resolve_callback_surface_size_for_kind(
            width,
            height,
            self.welcome_recent_pointer_size,
            ViewContentKind::Welcome,
        );
        let welcome = self.runtime.chrome_snapshot().welcome;
        self.sync_welcome_recent_pointer_layout(&welcome);
        match self
            .welcome_recent_pointer_bridge
            .handle_move(UiPoint::new(x, y))
        {
            Ok(dispatch) => {
                self.welcome_recent_pointer_state = dispatch.state;
                self.apply_welcome_recent_pointer_state_to_ui();
            }
            Err(error) => self.set_status_line(error),
        }
    }

    pub(super) fn welcome_recent_pointer_scrolled(
        &mut self,
        x: f32,
        y: f32,
        delta: f32,
        width: f32,
        height: f32,
    ) {
        self.welcome_recent_pointer_size = self.resolve_callback_surface_size_for_kind(
            width,
            height,
            self.welcome_recent_pointer_size,
            ViewContentKind::Welcome,
        );
        let welcome = self.runtime.chrome_snapshot().welcome;
        self.sync_welcome_recent_pointer_layout(&welcome);
        match self
            .welcome_recent_pointer_bridge
            .handle_scroll(UiPoint::new(x, y), delta)
        {
            Ok(dispatch) => {
                self.welcome_recent_pointer_state = dispatch.state;
                self.apply_welcome_recent_pointer_state_to_ui();
            }
            Err(error) => self.set_status_line(error),
        }
    }
}
