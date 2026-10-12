use std::collections::BTreeSet;

use zircon_runtime::ui::surface::UiAuthoredGeometryPublication;
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{UiFrame, UiSize},
};

use super::constants::{DISMISS_NODE_ID, ROOT_NODE_ID};
use super::host_menu_pointer_bridge::HostMenuPointerBridge;
use super::host_menu_pointer_route_intent::HostMenuPointerRouteIntent;
use super::node_ids::{menu_button_node_id, popup_node_id};
use super::popup_layout::{
    clipped_menu_button_frame, popup_grid_layout, popup_item_frame, submenu_popup_grid_layout,
};
use crate::ui::retained_host::route_intent::EditorRouteIntent;

impl HostMenuPointerBridge {
    pub(in crate::ui::retained_host::menu_pointer) fn rebuild_surface(&mut self) {
        self.apply_surface_delta();
    }

    pub(in crate::ui::retained_host::menu_pointer) fn apply_surface_delta(&mut self) {
        let expected_nodes = self.expected_surface_nodes();
        match self.classify_surface_delta(&expected_nodes) {
            MenuPointerSurfaceDelta::NoChange => {
                zircon_runtime::profile_counter!(
                    "editor",
                    "menu_pointer.surface_delta_no_change_count",
                    1_u8,
                );
            }
            MenuPointerSurfaceDelta::Geometry(changes) => {
                self.apply_geometry_delta(changes);
            }
            MenuPointerSurfaceDelta::Topology => {
                zircon_runtime::profile_counter!(
                    "editor",
                    "menu_pointer.surface_delta_topology_count",
                    1_u8,
                );
                self.rebuild_surface_from_scratch();
            }
        }
    }

    fn classify_surface_delta(
        &self,
        expected_nodes: &[ExpectedSurfaceNode],
    ) -> MenuPointerSurfaceDelta {
        // Validate every retained node and route before mutating frames so fallback is atomic.
        if expected_nodes.len() != self.surface.tree.nodes.len() {
            return MenuPointerSurfaceDelta::Topology;
        }

        let mut changes = Vec::new();
        for expected in expected_nodes {
            let Some(node) = self.surface.tree.node(expected.node_id) else {
                return MenuPointerSurfaceDelta::Topology;
            };
            if !expected
                .route
                .matches(&self.route_intents, expected.node_id)
            {
                return MenuPointerSurfaceDelta::Topology;
            }
            if node.layout_cache.frame != expected.frame {
                changes.push(NodeFrameChange {
                    node_id: expected.node_id,
                    frame: expected.frame,
                });
            }
        }

        if changes.is_empty() {
            MenuPointerSurfaceDelta::NoChange
        } else {
            MenuPointerSurfaceDelta::Geometry(changes)
        }
    }

    fn apply_geometry_delta(&mut self, changes: Vec<NodeFrameChange>) {
        let observed_topology_generation = self.surface.tree.layout_order_generation();
        let mut changed_node_ids = BTreeSet::new();
        for change in changes {
            self.surface
                .tree
                .node_mut(change.node_id)
                .expect("menu geometry receipt must reference retained topology")
                .layout_cache
                .frame = change.frame;
            changed_node_ids.insert(change.node_id);
        }

        zircon_runtime::profile_counter!(
            "editor",
            "menu_pointer.surface_delta_geometry_count",
            1_u8,
        );
        zircon_runtime::profile_counter!(
            "editor",
            "menu_pointer.surface_geometry_patch_node_count",
            changed_node_ids.len(),
        );
        match self.surface.publish_authored_geometry(
            UiSize::new(
                self.layout.shell_frame.width,
                self.layout.shell_frame.height,
            ),
            &changed_node_ids,
            observed_topology_generation,
        ) {
            UiAuthoredGeometryPublication::Local(_) => zircon_runtime::profile_counter!(
                "editor",
                "menu_pointer.surface_geometry_local_publication_count",
                1_u8,
            ),
            UiAuthoredGeometryPublication::FullFallback { .. } => {
                zircon_runtime::profile_counter!(
                    "editor",
                    "menu_pointer.surface_geometry_fallback_count",
                    1_u8,
                );
            }
            UiAuthoredGeometryPublication::Unchanged => {}
        }
    }

    fn expected_surface_nodes(&self) -> Vec<ExpectedSurfaceNode> {
        let mut nodes = Vec::with_capacity(
            self.layout
                .button_frames
                .len()
                .saturating_add(self.state.open_submenu_path.len())
                .saturating_add(3),
        );
        nodes.push(ExpectedSurfaceNode::new(
            ROOT_NODE_ID,
            self.layout.shell_frame,
            ExpectedRoute::None,
        ));

        for menu_index in 0..self.layout.button_frames.len() {
            let Some(frame) = clipped_menu_button_frame(
                &self.layout,
                menu_index,
                self.state.menu_bar_scroll_offset,
            ) else {
                continue;
            };
            nodes.push(ExpectedSurfaceNode::new(
                menu_button_node_id(menu_index),
                frame,
                ExpectedRoute::MenuButton(menu_index),
            ));
        }

        let Some(menu_index) = self.state.open_menu_index else {
            return nodes;
        };
        nodes.push(ExpectedSurfaceNode::new(
            DISMISS_NODE_ID,
            self.layout.shell_frame,
            ExpectedRoute::Dismiss,
        ));

        let mut grid = popup_grid_layout(
            &self.layout,
            menu_index,
            self.popup_items.len(),
            self.state.popup_scroll_offset,
            self.state.menu_bar_scroll_offset,
        );
        nodes.push(ExpectedSurfaceNode::new(
            popup_node_id(0),
            grid.frame,
            ExpectedRoute::Popup(menu_index),
        ));
        let mut visible_items = self.popup_items.as_slice();
        for (level, selected_index) in self.state.open_submenu_path.iter().copied().enumerate() {
            let Some(branch_item) = visible_items
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
            nodes.push(ExpectedSurfaceNode::new(
                popup_node_id(level + 1),
                grid.frame,
                ExpectedRoute::Popup(menu_index),
            ));
            visible_items = branch_item.children.as_slice();
        }
        nodes
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ExpectedSurfaceNode {
    node_id: UiNodeId,
    frame: UiFrame,
    route: ExpectedRoute,
}

impl ExpectedSurfaceNode {
    const fn new(node_id: UiNodeId, frame: UiFrame, route: ExpectedRoute) -> Self {
        Self {
            node_id,
            frame,
            route,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExpectedRoute {
    None,
    MenuButton(usize),
    Dismiss,
    Popup(usize),
}

impl ExpectedRoute {
    fn matches(
        self,
        route_intents: &crate::ui::retained_host::route_intent::EditorRouteIntentMap,
        node_id: UiNodeId,
    ) -> bool {
        let actual = route_intents.intent_for_node(node_id);
        match (self, actual) {
            (Self::None, None) => true,
            (
                Self::MenuButton(expected),
                Some(EditorRouteIntent::Menu(HostMenuPointerRouteIntent::MenuButton(actual))),
            ) => expected == *actual,
            (
                Self::Dismiss,
                Some(EditorRouteIntent::Menu(HostMenuPointerRouteIntent::DismissOverlay)),
            ) => true,
            (
                Self::Popup(expected),
                Some(EditorRouteIntent::Menu(HostMenuPointerRouteIntent::PopupSurface(actual))),
            ) => expected == *actual,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct NodeFrameChange {
    node_id: UiNodeId,
    frame: UiFrame,
}

#[derive(Debug, PartialEq)]
enum MenuPointerSurfaceDelta {
    NoChange,
    Geometry(Vec<NodeFrameChange>),
    Topology,
}
