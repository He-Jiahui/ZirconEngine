use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodePath, UiTreeId},
    layout::{
        UiAxis, UiContainerKind, UiFrame, UiScrollState, UiScrollableBoxConfig,
        UiScrollbarVisibility,
    },
    tree::{UiInputPolicy, UiTreeNode},
};

use crate::ui::retained_host::route_intent::{EditorRouteIntent, EditorRouteIntentMap};

use super::base_state::base_state;
use super::constants::{ROOT_NODE_ID, VIEWPORT_NODE_ID};
use super::content_height::content_height;
use super::hierarchy_pointer_bridge::HierarchyPointerBridge;
use super::hierarchy_pointer_route::HierarchyPointerRoute;
use super::item_node_id::{item_node_id, item_route_id, list_surface_route_id};
use super::register_handled_pointer_node::register_handled_pointer_node;
use super::row_metrics::{hierarchy_row_width, hierarchy_row_y};
use super::viewport_frame::viewport_frame;

impl HierarchyPointerBridge {
    pub(super) fn rebuild_surface(&mut self) {
        let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.hierarchy.pointer"));
        let mut dispatcher = UiPointerDispatcher::default();
        let mut route_intents = EditorRouteIntentMap::default();

        surface.tree.insert_root(
            UiTreeNode::new(ROOT_NODE_ID, UiNodePath::new("editor.hierarchy.root"))
                .with_frame(UiFrame::new(
                    0.0,
                    0.0,
                    self.layout.pane_width.max(0.0),
                    self.layout.pane_height.max(0.0),
                ))
                .with_state_flags(base_state(false)),
        );

        let viewport = viewport_frame(&self.layout);
        surface
            .tree
            .insert_child(
                ROOT_NODE_ID,
                UiTreeNode::new(
                    VIEWPORT_NODE_ID,
                    UiNodePath::new("editor.hierarchy.viewport"),
                )
                .with_frame(viewport)
                .with_z_index(10)
                .with_input_policy(UiInputPolicy::Receive)
                .with_clip_to_bounds(true)
                .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                    axis: UiAxis::Vertical,
                    gap: 0.0,
                    scrollbar_visibility: UiScrollbarVisibility::Auto,
                    virtualization: None,
                }))
                .with_scroll_state(UiScrollState {
                    offset: self.state.scroll_offset,
                    viewport_extent: viewport.height.max(0.0),
                    content_extent: content_height(self.layout.node_ids.len(), self.row_metrics),
                })
                .with_state_flags(base_state(true)),
            )
            .expect("hierarchy root must exist");
        register_handled_pointer_node(&mut dispatcher, VIEWPORT_NODE_ID);
        route_intents.bind_node(
            VIEWPORT_NODE_ID,
            list_surface_route_id(),
            EditorRouteIntent::Hierarchy(HierarchyPointerRoute::ListSurface),
        );

        let row_width = hierarchy_row_width(self.layout.pane_width, self.row_metrics);
        for (item_index, node_id) in self.layout.node_ids.iter().enumerate() {
            let item_node_id = item_node_id(item_index);
            surface
                .tree
                .insert_child(
                    VIEWPORT_NODE_ID,
                    UiTreeNode::new(
                        item_node_id,
                        UiNodePath::new(format!("editor.hierarchy/item_{item_index}")),
                    )
                    .with_frame(UiFrame::new(
                        self.row_metrics.row_x,
                        hierarchy_row_y(self.row_metrics, item_index, self.state.scroll_offset),
                        row_width,
                        self.row_metrics.row_height,
                    ))
                    .with_z_index(20 + item_index as i32)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_state_flags(base_state(true)),
                )
                .expect("hierarchy viewport must exist");
            register_handled_pointer_node(&mut dispatcher, item_node_id);
            route_intents.bind_node(
                item_node_id,
                item_route_id(item_index),
                EditorRouteIntent::Hierarchy(HierarchyPointerRoute::Node {
                    item_index,
                    node_id: node_id.clone(),
                }),
            );
        }

        surface.rebuild();
        self.surface = surface;
        self.dispatcher = dispatcher;
        self.route_intents = route_intents;
    }
}
