use super::super::super::{RetainedEditorHost, UiPoint};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn hierarchy_pointer_moved(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        let owner_move = self
            .hierarchy_input_owner
            .admits_move(&self.callback_source_window);
        self.prepare_hierarchy_pointer_target(width, height, false);
        let dispatch = self
            .hierarchy_pointer_bridge
            .handle_move(UiPoint::new(x, y));
        if owner_move {
            self.hierarchy_pointer_bridge
                .observe_reparent_drag_move(UiPoint::new(x, y));
        }
        self.hierarchy_pointer_state = dispatch.state;
        self.apply_hierarchy_pointer_state_to_ui();
    }
}
