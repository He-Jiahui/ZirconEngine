use super::workbench_drawer_header_pointer_route::WorkbenchDrawerHeaderPointerRoute;
use super::workbench_drawer_header_pointer_target::WorkbenchDrawerHeaderPointerTarget;

pub(super) fn to_public_route(
    target: WorkbenchDrawerHeaderPointerTarget,
) -> WorkbenchDrawerHeaderPointerRoute {
    match target {
        WorkbenchDrawerHeaderPointerTarget::Tab {
            surface_key,
            item_index,
            slot,
            instance_id,
        } => WorkbenchDrawerHeaderPointerRoute::Tab {
            surface_key,
            item_index,
            slot,
            instance_id,
        },
    }
}
