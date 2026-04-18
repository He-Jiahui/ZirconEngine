use super::workbench_activity_rail_pointer_route::WorkbenchActivityRailPointerRoute;
use super::workbench_activity_rail_pointer_target::WorkbenchActivityRailPointerTarget;

pub(super) fn to_public_route(
    target: WorkbenchActivityRailPointerTarget,
) -> WorkbenchActivityRailPointerRoute {
    match target {
        WorkbenchActivityRailPointerTarget::Button {
            side,
            item_index,
            slot,
            instance_id,
        } => WorkbenchActivityRailPointerRoute::Button {
            side,
            item_index,
            slot,
            instance_id,
        },
        WorkbenchActivityRailPointerTarget::Strip(side) => {
            WorkbenchActivityRailPointerRoute::Strip(side)
        }
    }
}
