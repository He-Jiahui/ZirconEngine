use super::host_activity_rail_pointer_route::HostActivityRailPointerRoute;
use super::host_activity_rail_pointer_target::HostActivityRailPointerTarget;

pub(super) fn to_public_route(
    target: HostActivityRailPointerTarget,
) -> HostActivityRailPointerRoute {
    match target {
        HostActivityRailPointerTarget::Button {
            side,
            item_index,
            slot,
            instance_id,
        } => HostActivityRailPointerRoute::Button {
            side,
            item_index,
            slot,
            instance_id,
        },
        HostActivityRailPointerTarget::Strip(side) => HostActivityRailPointerRoute::Strip(side),
    }
}
