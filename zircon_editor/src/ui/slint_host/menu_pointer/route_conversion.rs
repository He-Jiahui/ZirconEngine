use super::workbench_menu_pointer_route::WorkbenchMenuPointerRoute;
use super::workbench_menu_pointer_target::WorkbenchMenuPointerTarget;

pub(in crate::ui::slint_host::menu_pointer) fn to_public_route(
    target: WorkbenchMenuPointerTarget,
) -> WorkbenchMenuPointerRoute {
    match target {
        WorkbenchMenuPointerTarget::MenuButton(index) => {
            WorkbenchMenuPointerRoute::MenuButton(index)
        }
        WorkbenchMenuPointerTarget::MenuItem {
            menu_index,
            item_index,
            action_id,
        } => WorkbenchMenuPointerRoute::MenuItem {
            menu_index,
            item_index,
            action_id,
        },
        WorkbenchMenuPointerTarget::PopupSurface(menu_index) => {
            WorkbenchMenuPointerRoute::PopupSurface(menu_index)
        }
        WorkbenchMenuPointerTarget::DismissOverlay => WorkbenchMenuPointerRoute::DismissOverlay,
    }
}
