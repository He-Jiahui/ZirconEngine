use zircon_runtime_interface::ui::{
    dispatch::UiPointerEvent, layout::UiPoint, surface::UiPointerEventKind,
};

use super::host_page_pointer_bridge::HostPagePointerBridge;
use super::host_page_pointer_dispatch::HostPagePointerDispatch;
use super::host_page_pointer_route::HostPagePointerRoute;
use super::HostPagePointerError;

impl HostPagePointerBridge {
    pub(crate) fn handle_click(
        &mut self,
        item_index: usize,
        _tab_x: f32,
        _tab_width: f32,
        point: UiPoint,
    ) -> Result<HostPagePointerDispatch, HostPagePointerError> {
        let Some(callback_frame) = self
            .layout
            .tabs
            .iter()
            .find(|tab| tab.page_index == item_index)
            .map(|tab| tab.frame)
        else {
            return Ok(HostPagePointerDispatch {
                route: self.route_for_item(item_index),
            });
        };
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
