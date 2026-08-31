use super::super::super::{RetainedEditorHost, UiPoint};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn hierarchy_pointer_scrolled(
        &mut self,
        x: f32,
        y: f32,
        delta: f32,
        width: f32,
        height: f32,
    ) {
        self.prepare_hierarchy_pointer_target(width, height, true);
        let dispatch = self
            .hierarchy_pointer_bridge
            .handle_scroll(UiPoint::new(x, y), delta);
        self.hierarchy_pointer_state = dispatch.state;
        self.apply_hierarchy_pointer_state_to_ui();
    }
}
