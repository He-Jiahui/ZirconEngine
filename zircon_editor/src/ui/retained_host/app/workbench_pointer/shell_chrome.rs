use super::super::{callback_dispatch, HostActivityRailPointerSide, RetainedEditorHost};
use zircon_runtime_interface::ui::layout::UiPoint;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn activity_rail_pointer_clicked(
        &mut self,
        side: &str,
        x: f32,
        y: f32,
    ) {
        self.use_committed_pointer_layout();
        let side = match HostActivityRailPointerSide::parse(side) {
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

    pub(in crate::ui::retained_host::app) fn host_page_pointer_clicked(
        &mut self,
        tab_index: i32,
        tab_x: f32,
        tab_width: f32,
        point_x: f32,
        point_y: f32,
    ) {
        self.use_committed_pointer_layout();
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

    pub(in crate::ui::retained_host::app) fn drawer_header_pointer_clicked(
        &mut self,
        surface_key: &str,
        tab_index: i32,
        tab_x: f32,
        tab_width: f32,
        point_x: f32,
        point_y: f32,
    ) {
        self.use_committed_pointer_layout();
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
