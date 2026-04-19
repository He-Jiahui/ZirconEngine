use zircon_runtime::ui::{dispatch::UiPointerEvent, layout::UiPoint, surface::UiPointerEventKind};

use super::to_public_route::to_public_route;
use super::workbench_activity_rail_pointer_bridge::WorkbenchActivityRailPointerBridge;
use super::workbench_activity_rail_pointer_dispatch::WorkbenchActivityRailPointerDispatch;
use super::workbench_activity_rail_pointer_side::WorkbenchActivityRailPointerSide;
use super::workbench_activity_rail_pointer_target::WorkbenchActivityRailPointerTarget;

impl WorkbenchActivityRailPointerBridge {
    pub(crate) fn handle_click(
        &mut self,
        side: WorkbenchActivityRailPointerSide,
        point: UiPoint,
    ) -> Result<WorkbenchActivityRailPointerDispatch, String> {
        let local_point = self.global_point_for_side(side, point);
        let mut route =
            self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Down, local_point))?;
        if !matches!(
            route,
            Some(WorkbenchActivityRailPointerTarget::Button { .. })
        ) {
            let projected_route =
                self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Down, point))?;
            if matches!(
                projected_route,
                Some(WorkbenchActivityRailPointerTarget::Button { .. })
            ) || route.is_none()
            {
                route = projected_route;
            }
        }
        Ok(WorkbenchActivityRailPointerDispatch {
            route: route.map(to_public_route),
        })
    }
}
