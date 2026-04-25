use zircon_runtime::ui::{dispatch::UiPointerEvent, layout::UiPoint, surface::UiPointerEventKind};

use super::host_drawer_header_pointer_bridge::HostDrawerHeaderPointerBridge;
use super::host_drawer_header_pointer_dispatch::HostDrawerHeaderPointerDispatch;
use super::to_public_route::to_public_route;

impl HostDrawerHeaderPointerBridge {
    pub(crate) fn handle_click(
        &mut self,
        surface_key: &str,
        item_index: usize,
        tab_x: f32,
        tab_width: f32,
        point: UiPoint,
    ) -> Result<HostDrawerHeaderPointerDispatch, String> {
        self.update_measured_frame(surface_key, item_index, tab_x, tab_width)?;
        let point = self.global_point(surface_key, point)?;
        let route = self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Down, point))?;
        Ok(HostDrawerHeaderPointerDispatch {
            route: route.map(to_public_route),
        })
    }
}
