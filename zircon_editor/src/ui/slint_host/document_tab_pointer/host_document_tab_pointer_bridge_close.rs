use zircon_runtime_interface::ui::{
    dispatch::UiPointerEvent, layout::UiPoint, surface::UiPointerEventKind,
};

use super::host_document_tab_pointer_bridge::HostDocumentTabPointerBridge;
use super::host_document_tab_pointer_dispatch::HostDocumentTabPointerDispatch;
use super::route_conversion::to_public_route;

impl HostDocumentTabPointerBridge {
    pub(crate) fn handle_close_click(
        &mut self,
        surface_key: &str,
        item_index: usize,
        tab_x: f32,
        tab_width: f32,
        point: UiPoint,
    ) -> Result<HostDocumentTabPointerDispatch, String> {
        self.update_measured_frame(surface_key, item_index, tab_x, tab_width)?;
        let point = self.global_point(surface_key, point)?;
        let route = self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Down, point))?;
        Ok(HostDocumentTabPointerDispatch {
            route: route.map(to_public_route),
        })
    }
}
