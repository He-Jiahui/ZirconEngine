use super::host_menu_pointer_route::HostMenuPointerRoute;
use super::host_menu_pointer_target::HostMenuPointerTarget;

pub(in crate::ui::slint_host::menu_pointer) fn to_public_route(
    target: HostMenuPointerTarget,
) -> HostMenuPointerRoute {
    match target {
        HostMenuPointerTarget::MenuButton(index) => HostMenuPointerRoute::MenuButton(index),
        HostMenuPointerTarget::SubmenuBranch {
            menu_index,
            item_index,
            ..
        } => HostMenuPointerRoute::SubmenuBranch {
            menu_index,
            item_index,
        },
        HostMenuPointerTarget::MenuItem {
            menu_index,
            item_index,
            action_id,
            ..
        } => HostMenuPointerRoute::MenuItem {
            menu_index,
            item_index,
            action_id,
        },
        HostMenuPointerTarget::PopupSurface(menu_index) => {
            HostMenuPointerRoute::PopupSurface(menu_index)
        }
        HostMenuPointerTarget::DismissOverlay => HostMenuPointerRoute::DismissOverlay,
    }
}
