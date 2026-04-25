use super::host_page_pointer_route::HostPagePointerRoute;
use super::host_page_pointer_target::HostPagePointerTarget;

pub(super) fn to_public_route(target: HostPagePointerTarget) -> HostPagePointerRoute {
    match target {
        HostPagePointerTarget::Tab {
            item_index,
            page_id,
        } => HostPagePointerRoute::Tab {
            item_index,
            page_id,
        },
    }
}
