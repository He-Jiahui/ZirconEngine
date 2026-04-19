use zircon_ui::{dispatch::UiPointerEvent, UiPoint, UiPointerEventKind};

use super::route_conversion::to_public_route;
use super::workbench_document_tab_pointer_bridge::WorkbenchDocumentTabPointerBridge;
use super::workbench_document_tab_pointer_dispatch::WorkbenchDocumentTabPointerDispatch;

impl WorkbenchDocumentTabPointerBridge {
    pub(crate) fn handle_close_click(
        &mut self,
        surface_key: &str,
        item_index: usize,
        tab_x: f32,
        tab_width: f32,
        point: UiPoint,
    ) -> Result<WorkbenchDocumentTabPointerDispatch, String> {
        self.update_measured_frame(surface_key, item_index, tab_x, tab_width)?;
        let point = self.global_point(surface_key, point)?;
        let route = self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Down, point))?;
        Ok(WorkbenchDocumentTabPointerDispatch {
            route: route.map(to_public_route),
        })
    }
}
