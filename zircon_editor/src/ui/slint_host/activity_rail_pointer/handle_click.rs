use zircon_runtime_interface::ui::{
    dispatch::UiPointerEvent, layout::UiPoint, surface::UiPointerEventKind,
};

use super::host_activity_rail_pointer_bridge::HostActivityRailPointerBridge;
use super::host_activity_rail_pointer_dispatch::HostActivityRailPointerDispatch;
use super::host_activity_rail_pointer_side::HostActivityRailPointerSide;
use super::host_activity_rail_pointer_target::HostActivityRailPointerTarget;
use super::to_public_route::to_public_route;

impl HostActivityRailPointerBridge {
    pub(crate) fn handle_click(
        &mut self,
        side: HostActivityRailPointerSide,
        point: UiPoint,
    ) -> Result<HostActivityRailPointerDispatch, String> {
        let local_point = self.global_point_for_side(side, point);
        let mut route =
            self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Down, local_point))?;
        if !matches!(route, Some(HostActivityRailPointerTarget::Button { .. })) {
            let projected_route =
                self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Down, point))?;
            if matches!(
                projected_route,
                Some(HostActivityRailPointerTarget::Button { .. })
            ) || route.is_none()
            {
                route = projected_route;
            }
        }
        Ok(HostActivityRailPointerDispatch {
            route: route.map(to_public_route),
        })
    }
}
