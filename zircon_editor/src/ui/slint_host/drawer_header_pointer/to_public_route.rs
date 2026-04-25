use super::host_drawer_header_pointer_route::HostDrawerHeaderPointerRoute;
use super::host_drawer_header_pointer_target::HostDrawerHeaderPointerTarget;

pub(super) fn to_public_route(
    target: HostDrawerHeaderPointerTarget,
) -> HostDrawerHeaderPointerRoute {
    match target {
        HostDrawerHeaderPointerTarget::Tab {
            surface_key,
            item_index,
            slot,
            instance_id,
        } => HostDrawerHeaderPointerRoute::Tab {
            surface_key,
            item_index,
            slot,
            instance_id,
        },
    }
}
