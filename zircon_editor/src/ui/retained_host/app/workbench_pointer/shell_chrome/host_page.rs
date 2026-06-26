use super::super::super::{callback_dispatch, RetainedEditorHost};
use crate::ui::retained_host::{
    host_page_pointer::HOST_PAGE_OVERFLOW_POINTER_INDEX, HostPageOverflowMenuStateData,
    UiHostContext,
};
use zircon_runtime_interface::ui::layout::UiPoint;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn host_page_pointer_clicked(
        &mut self,
        tab_index: i32,
        tab_x: f32,
        tab_width: f32,
        point_x: f32,
        point_y: f32,
    ) {
        self.use_committed_pointer_layout();
        if tab_index == HOST_PAGE_OVERFLOW_POINTER_INDEX {
            self.host_page_overflow_pointer_clicked(point_x, point_y);
            return;
        }
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
                if dispatch.pointer.route.is_some() {
                    self.set_host_page_overflow_menu_open(false);
                }
                if let Some(effects) = dispatch.effects {
                    self.apply_dispatch_effects(effects);
                }
            }
            Err(error) => self.set_status_line(error),
        }
    }

    fn host_page_overflow_pointer_clicked(&mut self, point_x: f32, point_y: f32) {
        match callback_dispatch::dispatch_shared_host_page_overflow_pointer_click(
            &mut self.host_page_pointer_bridge,
            UiPoint::new(point_x, point_y),
        ) {
            Ok(dispatch) => {
                if dispatch.pointer.route.is_some() {
                    let open = !self
                        .ui
                        .get_host_presentation()
                        .host_page_overflow_menu_state
                        .open;
                    self.set_host_page_overflow_menu_open(open);
                }
                if let Some(effects) = dispatch.effects {
                    self.apply_dispatch_effects(effects);
                }
            }
            Err(error) => self.set_status_line(error),
        }
    }

    fn set_host_page_overflow_menu_open(&self, open: bool) {
        self.ui
            .global::<UiHostContext>()
            .set_host_page_overflow_menu_state(HostPageOverflowMenuStateData {
                open,
                hovered_page_index: -1,
            });
    }
}
