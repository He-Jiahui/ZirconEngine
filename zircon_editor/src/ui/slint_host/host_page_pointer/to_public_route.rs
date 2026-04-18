use super::workbench_host_page_pointer_route::WorkbenchHostPagePointerRoute;
use super::workbench_host_page_pointer_target::WorkbenchHostPagePointerTarget;

pub(super) fn to_public_route(
    target: WorkbenchHostPagePointerTarget,
) -> WorkbenchHostPagePointerRoute {
    match target {
        WorkbenchHostPagePointerTarget::Tab {
            item_index,
            page_id,
        } => WorkbenchHostPagePointerRoute::Tab {
            item_index,
            page_id,
        },
    }
}
