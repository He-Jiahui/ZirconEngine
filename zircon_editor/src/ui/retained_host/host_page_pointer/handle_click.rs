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
        tab_x: f32,
        tab_width: f32,
        point: UiPoint,
    ) -> Result<HostPagePointerDispatch, HostPagePointerError> {
        let Some(callback_frame) = self.update_measured_frame(item_index, tab_x, tab_width)? else {
            return Ok(HostPagePointerDispatch {
                route: self.route_for_item(item_index),
            });
        };
        let point = UiPoint::new(callback_frame.x + point.x, callback_frame.y + point.y);
        let route = self
            .dispatch_event(UiPointerEvent::new(UiPointerEventKind::Down, point))?
            .filter(|route| route_targets_item(route, item_index))
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

fn route_targets_item(route: &HostPagePointerRoute, item_index: usize) -> bool {
    match route {
        HostPagePointerRoute::Tab {
            item_index: route_index,
            ..
        }
        | HostPagePointerRoute::Close {
            item_index: route_index,
            ..
        } => *route_index == item_index,
        HostPagePointerRoute::Overflow { .. } => false,
    }
}
