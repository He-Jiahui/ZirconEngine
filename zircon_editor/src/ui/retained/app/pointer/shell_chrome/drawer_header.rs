use super::super::super::{callback_dispatch, RetainedEditorHost};
impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn drawer_header_pointer_clicked(
        &mut self,
        surface_key: &str,
        tab_index: i32,
        _tab_x: f32,
        _tab_width: f32,
        _point_x: f32,
        _point_y: f32,
    ) {
        self.use_committed_pointer_layout();
        if tab_index < 0 {
            self.set_status_line(format!("Invalid drawer header index {tab_index}"));
            return;
        }
        match callback_dispatch::dispatch_shared_drawer_header_pointer_click(
            &self.runtime,
            &self.drawer_header_pointer_bridge,
            surface_key,
            tab_index as usize,
        ) {
            Ok(dispatch) => {
                if let Some(effects) = dispatch.effects {
                    self.apply_dispatch_effects(effects);
                }
            }
            Err(error) => self.set_status_line(error),
        }
    }
}
