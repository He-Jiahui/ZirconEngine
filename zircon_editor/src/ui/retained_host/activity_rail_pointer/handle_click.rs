use zircon_runtime_interface::ui::{
    dispatch::UiPointerEvent, layout::UiPoint, surface::UiPointerEventKind,
};

use super::host_activity_rail_pointer_bridge::HostActivityRailPointerBridge;
use super::host_activity_rail_pointer_dispatch::HostActivityRailPointerDispatch;
use super::host_activity_rail_pointer_side::HostActivityRailPointerSide;

impl HostActivityRailPointerBridge {
    pub(crate) fn handle_click(
        &mut self,
        side: HostActivityRailPointerSide,
        point: UiPoint,
    ) -> Result<HostActivityRailPointerDispatch, String> {
        let global_point = self.global_point_for_side(side, point);
        let route =
            self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Down, global_point))?;
        Ok(HostActivityRailPointerDispatch { route })
    }

    #[cfg(test)]
    pub(crate) fn handle_click_at_global_point(
        &mut self,
        point: UiPoint,
    ) -> Result<HostActivityRailPointerDispatch, String> {
        let route = self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Down, point))?;
        Ok(HostActivityRailPointerDispatch { route })
    }
}
