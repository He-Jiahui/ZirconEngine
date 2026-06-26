use zircon_runtime_interface::ui::{
    dispatch::UiPointerEvent,
    layout::{UiFrame, UiPoint},
    surface::UiPointerEventKind,
};

use crate::ui::workbench::page_tabs::{
    MAIN_PAGE_TAB_HEIGHT, MAIN_PAGE_TAB_MAX_WIDTH, MAIN_PAGE_TAB_MIN_WIDTH, MAIN_PAGE_TAB_STRIP_Y,
};

use super::host_page_pointer_bridge::HostPagePointerBridge;
use super::host_page_pointer_dispatch::HostPagePointerDispatch;
use super::host_page_pointer_route::HostPagePointerRoute;
use super::HostPagePointerError;

impl HostPagePointerBridge {
    pub(crate) fn handle_click(
        &mut self,
        item_index: usize,
        tab_x: f32,
        tab_width: f32,
        point: UiPoint,
    ) -> Result<HostPagePointerDispatch, HostPagePointerError> {
        if !self
            .layout
            .tabs
            .iter()
            .any(|tab| tab.page_index == item_index)
        {
            return Ok(HostPagePointerDispatch {
                route: self.route_for_item(item_index),
            });
        }
        let callback_frame = self
            .layout
            .tabs
            .iter()
            .find(|tab| tab.page_index == item_index)
            .map(|tab| tab.frame)
            .unwrap_or_else(|| {
                UiFrame::new(
                    self.layout.strip_frame.x + tab_x,
                    self.layout.strip_frame.y + MAIN_PAGE_TAB_STRIP_Y,
                    tab_width.clamp(MAIN_PAGE_TAB_MIN_WIDTH, MAIN_PAGE_TAB_MAX_WIDTH),
                    MAIN_PAGE_TAB_HEIGHT,
                )
            });
        if item_index < self.measured_frames.len() {
            self.measured_frames[item_index] = Some(callback_frame);
        }
        if self.layout.tabs.is_empty() {
            self.rebuild_surface();
        }
        let point = UiPoint::new(callback_frame.x + point.x, callback_frame.y + point.y);
        let route = self
            .dispatch_event(UiPointerEvent::new(UiPointerEventKind::Down, point))?
            .or_else(|| self.route_for_item(item_index));
        Ok(HostPagePointerDispatch { route })
    }

    pub(super) fn route_for_item(&self, item_index: usize) -> Option<HostPagePointerRoute> {
        self.layout
            .items
            .get(item_index)
            .map(|item| HostPagePointerRoute::Tab {
                item_index,
                page_id: item.page_id.clone(),
            })
    }
}
