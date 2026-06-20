use super::super::super::{callback_dispatch, RetainedEditorHost, UiPoint};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn hierarchy_pointer_clicked(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        self.prepare_hierarchy_pointer_target(width, height, true);
        match callback_dispatch::dispatch_shared_hierarchy_pointer_click(
            &self.runtime,
            &mut self.hierarchy_pointer_bridge,
            UiPoint::new(x, y),
        ) {
            Ok(dispatch) => {
                self.hierarchy_pointer_state = dispatch.pointer.state;
                self.apply_hierarchy_pointer_state_to_ui();
                if let Some(effects) = dispatch.effects {
                    self.apply_dispatch_effects(effects);
                }
            }
            Err(error) => self.set_status_line(error),
        }
    }
}
