use zircon_ui::{UiPoint, UiPointerEvent, UiPointerEventKind};

use super::to_public_route::to_public_route;
use super::workbench_drawer_header_pointer_bridge::WorkbenchDrawerHeaderPointerBridge;
use super::workbench_drawer_header_pointer_dispatch::WorkbenchDrawerHeaderPointerDispatch;

impl WorkbenchDrawerHeaderPointerBridge {
    pub(crate) fn handle_click(
        &mut self,
        surface_key: &str,
        item_index: usize,
        tab_x: f32,
        tab_width: f32,
        point: UiPoint,
    ) -> Result<WorkbenchDrawerHeaderPointerDispatch, String> {
        self.update_measured_frame(surface_key, item_index, tab_x, tab_width)?;
        let point = self.global_point(surface_key, point)?;
        let route = self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Down, point))?;
        Ok(WorkbenchDrawerHeaderPointerDispatch {
            route: route.map(to_public_route),
        })
    }
}
