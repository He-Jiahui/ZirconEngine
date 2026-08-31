use super::super::{callback_dispatch, RetainedEditorHost};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn document_tab_pointer_clicked(
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
            self.set_status_line(format!("Invalid document tab index {tab_index}"));
            return;
        }
        match callback_dispatch::dispatch_shared_document_tab_pointer_click(
            &self.runtime,
            &self.template_bridge,
            &self.document_tab_pointer_bridge,
            surface_key,
            tab_index as usize,
        ) {
            Ok(dispatch) => {
                if let Some(effects) = dispatch.effects {
                    self.apply_dispatch_effects(effects);
                }
                self.note_focused_floating_window_surface(surface_key);
            }
            Err(error) => self.set_status_line(error),
        }
    }

    pub(in crate::ui::retained_host::app) fn document_tab_close_pointer_clicked(
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
            self.set_status_line(format!("Invalid document tab close index {tab_index}"));
            return;
        }
        match callback_dispatch::dispatch_shared_document_tab_close_pointer_click(
            &self.runtime,
            &self.template_bridge,
            &self.document_tab_pointer_bridge,
            surface_key,
            tab_index as usize,
        ) {
            Ok(dispatch) => {
                if let Some(effects) = dispatch.effects {
                    self.apply_dispatch_effects(effects);
                }
                self.note_focused_floating_window_surface(surface_key);
            }
            Err(error) => self.set_status_line(error),
        }
    }
}
