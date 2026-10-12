use super::host_menu_pointer_route::HostMenuPointerRoute;
use super::host_menu_pointer_route_intent::HostMenuPointerRouteIntent;

pub(in crate::ui::retained_host::menu_pointer) fn to_public_route(
    target: HostMenuPointerRouteIntent,
) -> HostMenuPointerRoute {
    match target {
        HostMenuPointerRouteIntent::MenuButton(index) => HostMenuPointerRoute::MenuButton(index),
        HostMenuPointerRouteIntent::SubmenuBranch {
            menu_index,
            item_index,
            ..
        } => HostMenuPointerRoute::SubmenuBranch {
            menu_index,
            item_index,
        },
        HostMenuPointerRouteIntent::MenuItem {
            menu_index,
            item_index,
            action_id,
            ..
        } => HostMenuPointerRoute::MenuItem {
            menu_index,
            item_index,
            action_id,
        },
        HostMenuPointerRouteIntent::PopupSurface(menu_index) => {
            HostMenuPointerRoute::PopupSurface(menu_index)
        }
        HostMenuPointerRouteIntent::DismissOverlay => HostMenuPointerRoute::DismissOverlay,
    }
}
