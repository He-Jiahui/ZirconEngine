use zircon_runtime::ui::{
    dispatch::UiPointerEvent, layout::UiFrame, layout::UiPoint, surface::UiPointerEventKind,
};

use super::constants::{STRIP_Y, TAB_HEIGHT, TAB_MIN_WIDTH};
use super::host_page_pointer_bridge::HostPagePointerBridge;
use super::host_page_pointer_dispatch::HostPagePointerDispatch;
use super::to_public_route::to_public_route;

impl HostPagePointerBridge {
    pub(crate) fn handle_click(
        &mut self,
        item_index: usize,
        tab_x: f32,
        tab_width: f32,
        point: UiPoint,
    ) -> Result<HostPagePointerDispatch, String> {
        if item_index < self.measured_frames.len() {
            self.measured_frames[item_index] = Some(UiFrame::new(
                self.layout.strip_frame.x + tab_x,
                self.layout.strip_frame.y + STRIP_Y,
                tab_width.max(TAB_MIN_WIDTH),
                TAB_HEIGHT,
            ));
            self.rebuild_surface();
        }
        let point = UiPoint::new(
            self.layout.strip_frame.x + point.x,
            self.layout.strip_frame.y + point.y,
        );
        let route = self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Down, point))?;
        Ok(HostPagePointerDispatch {
            route: route.map(to_public_route),
        })
    }
}
