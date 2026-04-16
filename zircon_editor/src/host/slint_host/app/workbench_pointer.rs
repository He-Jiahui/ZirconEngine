use super::*;

impl SlintEditorHost {
    pub(super) fn activity_rail_pointer_clicked(&mut self, side: &str, x: f32, y: f32) {
        self.recompute_if_dirty();
        let side = match WorkbenchActivityRailPointerSide::parse(side) {
            Ok(side) => side,
            Err(error) => {
                self.set_status_line(error);
                return;
            }
        };
        match callback_dispatch::dispatch_shared_activity_rail_pointer_click(
            &self.runtime,
            &self.template_bridge,
            &mut self.activity_rail_pointer_bridge,
            side,
            UiPoint::new(x, y),
        ) {
            Ok(dispatch) => {
                if let Some(effects) = dispatch.effects {
                    self.apply_dispatch_effects(effects);
                }
            }
            Err(error) => self.set_status_line(error),
        }
    }

    pub(super) fn host_page_pointer_clicked(
        &mut self,
        tab_index: i32,
        tab_x: f32,
        tab_width: f32,
        point_x: f32,
        point_y: f32,
    ) {
        self.recompute_if_dirty();
        if tab_index < 0 {
            self.set_status_line(format!("Invalid host page tab index {tab_index}"));
            return;
        }
        match callback_dispatch::dispatch_shared_host_page_pointer_click(
            &self.runtime,
            &self.template_bridge,
            &mut self.host_page_pointer_bridge,
            tab_index as usize,
            tab_x,
            tab_width,
            UiPoint::new(point_x, point_y),
        ) {
            Ok(dispatch) => {
                if let Some(effects) = dispatch.effects {
                    self.apply_dispatch_effects(effects);
                }
            }
            Err(error) => self.set_status_line(error),
        }
    }

    pub(super) fn document_tab_pointer_clicked(
        &mut self,
        surface_key: &str,
        tab_index: i32,
        tab_x: f32,
        tab_width: f32,
        point_x: f32,
        point_y: f32,
    ) {
        self.recompute_if_dirty();
        if tab_index < 0 {
            self.set_status_line(format!("Invalid document tab index {tab_index}"));
            return;
        }
        match callback_dispatch::dispatch_shared_document_tab_pointer_click(
            &self.runtime,
            &self.template_bridge,
            &mut self.document_tab_pointer_bridge,
            surface_key,
            tab_index as usize,
            tab_x,
            tab_width,
            UiPoint::new(point_x, point_y),
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

    pub(super) fn document_tab_close_pointer_clicked(
        &mut self,
        surface_key: &str,
        tab_index: i32,
        tab_x: f32,
        tab_width: f32,
        point_x: f32,
        point_y: f32,
    ) {
        self.recompute_if_dirty();
        if tab_index < 0 {
            self.set_status_line(format!("Invalid document tab close index {tab_index}"));
            return;
        }
        match callback_dispatch::dispatch_shared_document_tab_close_pointer_click(
            &self.runtime,
            &self.template_bridge,
            &mut self.document_tab_pointer_bridge,
            surface_key,
            tab_index as usize,
            tab_x,
            tab_width,
            UiPoint::new(point_x, point_y),
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

    pub(super) fn floating_window_header_pointer_clicked(&mut self, x: f32, y: f32) {
        self.recompute_if_dirty();
        let Some(window_id) = self
            .shell_pointer_bridge
            .drag_route_at(UiPoint::new(x, y))
            .and_then(|route| match route {
                WorkbenchShellPointerRoute::FloatingWindow(window_id)
                | WorkbenchShellPointerRoute::FloatingWindowEdge { window_id, .. } => {
                    Some(window_id)
                }
                WorkbenchShellPointerRoute::DragTarget(_)
                | WorkbenchShellPointerRoute::DocumentEdge(_)
                | WorkbenchShellPointerRoute::Resize(_) => None,
            })
        else {
            return;
        };

        if let Some(result) =
            callback_dispatch::dispatch_builtin_floating_window_focus(&self.runtime, &window_id)
        {
            self.apply_dispatch_result(result);
            self.note_focused_floating_window(Some(window_id));
        }
    }

    pub(super) fn drawer_header_pointer_clicked(
        &mut self,
        surface_key: &str,
        tab_index: i32,
        tab_x: f32,
        tab_width: f32,
        point_x: f32,
        point_y: f32,
    ) {
        self.recompute_if_dirty();
        if tab_index < 0 {
            self.set_status_line(format!("Invalid drawer header index {tab_index}"));
            return;
        }
        match callback_dispatch::dispatch_shared_drawer_header_pointer_click(
            &self.runtime,
            &self.template_bridge,
            &mut self.drawer_header_pointer_bridge,
            surface_key,
            tab_index as usize,
            tab_x,
            tab_width,
            UiPoint::new(point_x, point_y),
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
