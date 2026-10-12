use super::*;

impl RetainedEditorHost {
    pub(super) fn menu_pointer_clicked(&mut self, x: f32, y: f32) {
        self.use_committed_pointer_layout();
        match callback_dispatch::dispatch_shared_menu_pointer_click(
            &self.runtime,
            &self.template_bridge,
            &mut self.menu_pointer_bridge,
            UiPoint::new(x, y),
        ) {
            Ok(dispatch) => {
                self.apply_menu_pointer_dispatch_state(dispatch.pointer.state);
                if let Some(effects) = dispatch.effects {
                    self.apply_dispatch_effects(effects);
                }
            }
            Err(error) => self.set_status_line(error),
        }
    }

    pub(super) fn menu_pointer_moved(&mut self, x: f32, y: f32) {
        self.use_committed_pointer_layout();
        match self.menu_pointer_bridge.handle_move(UiPoint::new(x, y)) {
            Ok(dispatch) => {
                self.apply_menu_pointer_dispatch_state(dispatch.state);
            }
            Err(error) => self.set_status_line(error),
        }
    }

    pub(super) fn menu_pointer_scrolled(&mut self, x: f32, y: f32, delta: f32) {
        self.use_committed_pointer_layout();
        match self
            .menu_pointer_bridge
            .handle_scroll(UiPoint::new(x, y), delta)
        {
            Ok(dispatch) => {
                self.apply_menu_pointer_dispatch_state(dispatch.state);
            }
            Err(error) => self.set_status_line(error),
        }
    }

    fn apply_menu_pointer_dispatch_state(&mut self, state: HostMenuPointerState) {
        if self.menu_pointer_state == state {
            return;
        }
        self.menu_pointer_state = state;
        self.apply_menu_pointer_state_to_ui();
    }
}
