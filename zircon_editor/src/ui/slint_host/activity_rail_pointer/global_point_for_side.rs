use zircon_ui::UiPoint;

use super::workbench_activity_rail_pointer_bridge::WorkbenchActivityRailPointerBridge;
use super::workbench_activity_rail_pointer_side::WorkbenchActivityRailPointerSide;

impl WorkbenchActivityRailPointerBridge {
    pub(super) fn global_point_for_side(
        &self,
        side: WorkbenchActivityRailPointerSide,
        point: UiPoint,
    ) -> UiPoint {
        let frame = match side {
            WorkbenchActivityRailPointerSide::Left => self.layout.left_strip_frame,
            WorkbenchActivityRailPointerSide::Right => self.layout.right_strip_frame,
        };
        UiPoint::new(frame.x + point.x, frame.y + point.y)
    }
}
