use zircon_runtime_interface::ui::layout::UiPoint;

use super::host_activity_rail_pointer_bridge::HostActivityRailPointerBridge;
use super::host_activity_rail_pointer_side::HostActivityRailPointerSide;

impl HostActivityRailPointerBridge {
    pub(super) fn global_point_for_side(
        &self,
        side: HostActivityRailPointerSide,
        point: UiPoint,
    ) -> UiPoint {
        let frame = match side {
            HostActivityRailPointerSide::Left => self.layout.left_strip_frame,
            HostActivityRailPointerSide::Right => self.layout.right_strip_frame,
        };
        UiPoint::new(frame.x + point.x, frame.y + point.y)
    }
}
