use std::collections::HashMap;

use zircon_runtime_interface::ui::layout::UiPoint;

use super::host_menu_pointer_bridge::HostMenuPointerBridge;
use super::host_menu_pointer_layout::HostMenuPointerLayout;
use super::host_menu_pointer_route_intent::HostMenuPointerRouteIntent;
use super::menu_item_spec::MenuItemSpec;
use super::popup_layout::{
    popup_grid_layout, popup_item_frame, popup_item_index_at_point, submenu_popup_grid_layout,
    PopupGridLayout,
};

impl HostMenuPointerBridge {
    pub(super) fn project_route_at_point(
        &self,
        dispatched_route: Option<HostMenuPointerRouteIntent>,
        point: UiPoint,
    ) -> Option<HostMenuPointerRouteIntent> {
        match dispatched_route {
            Some(HostMenuPointerRouteIntent::PopupSurface(menu_index))
                if self.state.open_menu_index == Some(menu_index) =>
            {
                self.project_open_popup_route_at_point(point)
                    .or(Some(HostMenuPointerRouteIntent::PopupSurface(menu_index)))
            }
            route => route,
        }
    }

    pub(super) fn project_open_popup_route_at_point(
        &self,
        point: UiPoint,
    ) -> Option<HostMenuPointerRouteIntent> {
        let menu_index = self.state.open_menu_index?;
        let root_grid = popup_grid_layout(
            &self.layout,
            menu_index,
            self.popup_items.len(),
            self.state.popup_scroll_offset,
            self.state.menu_bar_scroll_offset,
        );
        project_popup_layer(PopupProjection {
            layout: &self.layout,
            menu_index,
            items: self.popup_items.as_slice(),
            route_indices: &self.popup_route_indices,
            open_submenu_path: self.state.open_submenu_path.as_slice(),
            item_path: &mut Vec::new(),
            grid: root_grid,
            point,
        })
    }

    pub(super) fn root_popup_accepts_scroll(&self, point: UiPoint) -> bool {
        let Some(menu_index) = self.state.open_menu_index else {
            return false;
        };
        let mut grid = popup_grid_layout(
            &self.layout,
            menu_index,
            self.popup_items.len(),
            self.state.popup_scroll_offset,
            self.state.menu_bar_scroll_offset,
        );
        if !grid.frame.contains_point(point) {
            return false;
        }
        let mut items = self.popup_items.as_slice();
        for selected_index in self.state.open_submenu_path.iter().copied() {
            let Some(branch_item) = items
                .get(selected_index)
                .filter(|item| item.enabled && item.has_children())
            else {
                break;
            };
            grid = submenu_popup_grid_layout(
                &self.layout,
                popup_item_frame(grid, selected_index),
                branch_item.children.len(),
            );
            if grid.frame.contains_point(point) {
                return false;
            }
            items = branch_item.children.as_slice();
        }
        true
    }
}

struct PopupProjection<'a> {
    layout: &'a HostMenuPointerLayout,
    menu_index: usize,
    items: &'a [MenuItemSpec],
    route_indices: &'a HashMap<Vec<usize>, usize>,
    open_submenu_path: &'a [usize],
    item_path: &'a mut Vec<usize>,
    grid: PopupGridLayout,
    point: UiPoint,
}

fn project_popup_layer(args: PopupProjection<'_>) -> Option<HostMenuPointerRouteIntent> {
    let level = args.item_path.len();
    if let Some(selected_index) = args.open_submenu_path.get(level).copied() {
        if let Some(branch_item) = args
            .items
            .get(selected_index)
            .filter(|item| item.enabled && item.has_children())
        {
            let child_grid = submenu_popup_grid_layout(
                args.layout,
                popup_item_frame(args.grid, selected_index),
                branch_item.children.len(),
            );
            args.item_path.push(selected_index);
            let child_route = project_popup_layer(PopupProjection {
                layout: args.layout,
                menu_index: args.menu_index,
                items: branch_item.children.as_slice(),
                route_indices: args.route_indices,
                open_submenu_path: args.open_submenu_path,
                item_path: &mut *args.item_path,
                grid: child_grid,
                point: args.point,
            });
            args.item_path.pop();
            if child_route.is_some() {
                return child_route;
            }
        }
    }

    if !args.grid.frame.contains_point(args.point) {
        return None;
    }
    let Some(visible_index) = popup_item_index_at_point(args.grid, args.items.len(), args.point)
    else {
        return Some(HostMenuPointerRouteIntent::PopupSurface(args.menu_index));
    };
    let item = &args.items[visible_index];
    args.item_path.push(visible_index);
    let item_index = args.route_indices.get(args.item_path.as_slice()).copied();
    let item_path = args.item_path.clone();
    args.item_path.pop();
    let Some(item_index) = item_index else {
        return Some(HostMenuPointerRouteIntent::PopupSurface(args.menu_index));
    };

    if item.enabled && item.has_children() {
        return Some(HostMenuPointerRouteIntent::SubmenuBranch {
            menu_index: args.menu_index,
            item_index,
            item_path,
        });
    }
    if item.enabled {
        if let Some(action_id) = item.action_id.clone() {
            return Some(HostMenuPointerRouteIntent::MenuItem {
                menu_index: args.menu_index,
                item_index,
                item_path,
                action_id,
            });
        }
    }
    Some(HostMenuPointerRouteIntent::PopupSurface(args.menu_index))
}
