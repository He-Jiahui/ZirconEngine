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
        let mut item_path = popup_item_path(self.state.open_submenu_path.as_slice());
        project_popup_layer(PopupProjection {
            layout: &self.layout,
            menu_index,
            items: self.popup_items.as_slice(),
            route_indices: &self.popup_route_indices,
            open_submenu_path: self.state.open_submenu_path.as_slice(),
            item_path: &mut item_path,
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

fn popup_item_path(open_submenu_path: &[usize]) -> Vec<usize> {
    Vec::with_capacity(open_submenu_path.len().saturating_add(1))
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

#[cfg(test)]
mod optimization_tests {
    use super::popup_item_path;

    #[test]
    fn optimization_batch_20260830cx_popup_item_path_reserves_open_depth_and_hit() {
        let path = popup_item_path(&[2, 4, 1]);

        assert!(path.is_empty());
        assert!(path.capacity() >= 4);
    }

    #[test]
    fn optimization_batch_20260830cx_popup_item_path_capacity_source_contract() {
        let source = include_str!("host_menu_pointer_bridge_project_route.rs");
        assert!(source.contains("let mut item_path = popup_item_path("));
        assert!(source.contains("Vec::with_capacity(open_submenu_path.len().saturating_add(1))"));
        assert!(!source.contains("item_path: &mut Vec::new()"));
    }

    #[test]
    #[ignore = "release performance evidence; run through the validation coordinator"]
    fn optimization_batch_20260830cx_editor_popup_item_path_capacity_p95() {
        fn measure(open_path: &[usize], reserve: bool) -> u128 {
            let started = std::time::Instant::now();
            for _ in 0..16_384 {
                let mut path = if reserve {
                    popup_item_path(open_path)
                } else {
                    Vec::new()
                };
                for index in std::hint::black_box(open_path) {
                    path.push(*index);
                }
                path.push(open_path.len());
                std::hint::black_box(path);
            }
            started.elapsed().as_nanos()
        }

        let open_path = (0..24).collect::<Vec<_>>();
        let mut legacy_samples = Vec::with_capacity(17);
        let mut optimized_samples = Vec::with_capacity(17);
        for sample_index in 0..17 {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure(&open_path, false));
                optimized_samples.push(measure(&open_path, true));
            } else {
                optimized_samples.push(measure(&open_path, true));
                legacy_samples.push(measure(&open_path, false));
            }
        }

        legacy_samples.sort_unstable();
        optimized_samples.sort_unstable();
        let legacy_p95 = legacy_samples[16];
        let optimized_p95 = optimized_samples[16];
        println!(
            "EDITOR340_POPUP_ITEM_PATH_CAPACITY_BENCH_V1 depth={} legacy_p95_ns={} optimized_p95_ns={} target_ratio_bp=7000",
            open_path.len(),
            legacy_p95,
            optimized_p95,
        );
        assert!(
            optimized_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(7_000),
            "preallocated popup path P95 {optimized_p95} ns exceeded 70% of legacy {legacy_p95} ns"
        );
    }
}
