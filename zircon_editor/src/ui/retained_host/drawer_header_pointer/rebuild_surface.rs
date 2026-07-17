use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};
use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiNodePath, UiRouteId, UiTreeId};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::tree::{UiInputPolicy, UiTreeNode};

use super::base_state::base_state;
use super::constants::{
    DRAWER_HEADER_ROUTE_ID_BASE, ROOT_NODE_ID, STRIP_X, STRIP_Y, SURFACE_NODE_ID_BASE, TAB_GAP,
    TAB_HEIGHT, TAB_MIN_WIDTH, TAB_NODE_ID_BASE,
};
use super::host_drawer_header_pointer_bridge::HostDrawerHeaderPointerBridge;
use super::host_drawer_header_pointer_route::HostDrawerHeaderPointerRoute;
use super::register_handled_pointer_node::register_handled_pointer_node;
use super::root_frame::root_frame;
use crate::ui::retained_host::route_intent::{EditorRouteIntent, EditorRouteIntentMap};

impl HostDrawerHeaderPointerBridge {
    pub(super) fn rebuild_surface(&mut self) {
        let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.drawer_header.pointer"));
        let mut dispatcher = UiPointerDispatcher::default();
        let mut route_intents = EditorRouteIntentMap::default();

        surface.tree.insert_root(
            UiTreeNode::new(ROOT_NODE_ID, UiNodePath::new("editor.drawer_header.root"))
                .with_frame(root_frame(&self.layout))
                .with_state_flags(base_state(false)),
        );

        for (surface_index, surface_layout) in self.layout.surfaces.iter().enumerate() {
            let surface_node_id = UiNodeId::new(SURFACE_NODE_ID_BASE + surface_index as u64);
            surface
                .tree
                .insert_child(
                    ROOT_NODE_ID,
                    UiTreeNode::new(
                        surface_node_id,
                        UiNodePath::new(format!("editor.drawer_header/{}", surface_layout.key)),
                    )
                    .with_frame(surface_layout.strip_frame)
                    .with_z_index(10 + surface_index as i32)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_state_flags(base_state(true)),
                )
                .expect("drawer header root must exist");

            let measured = self.measured_frames.get(surface_layout.key.as_str());
            let mut next_x = surface_layout.strip_frame.x + STRIP_X;

            for (item_index, item) in surface_layout.items.iter().enumerate() {
                let frame = measured
                    .and_then(|frames| frames.get(item_index))
                    .copied()
                    .flatten()
                    .unwrap_or_else(|| {
                        UiFrame::new(
                            next_x,
                            surface_layout.strip_frame.y + STRIP_Y,
                            TAB_MIN_WIDTH,
                            TAB_HEIGHT,
                        )
                    });
                next_x = frame.x + frame.width + TAB_GAP;

                let node_id = UiNodeId::new(
                    TAB_NODE_ID_BASE + surface_index as u64 * 100 + item_index as u64,
                );
                surface
                    .tree
                    .insert_child(
                        surface_node_id,
                        UiTreeNode::new(
                            node_id,
                            UiNodePath::new(format!(
                                "editor.drawer_header/{}/tab_{item_index}",
                                surface_layout.key
                            )),
                        )
                        .with_frame(frame)
                        .with_z_index(20 + item_index as i32)
                        .with_input_policy(UiInputPolicy::Receive)
                        .with_state_flags(base_state(true)),
                    )
                    .expect("drawer header surface must exist");
                register_handled_pointer_node(&mut dispatcher, node_id);
                route_intents.bind_node(
                    node_id,
                    drawer_header_route_id(surface_index, item_index),
                    EditorRouteIntent::DrawerHeader(HostDrawerHeaderPointerRoute::Tab {
                        surface_key: surface_layout.key.clone(),
                        item_index,
                        slot: item.slot.clone(),
                        instance_id: item.instance_id.clone(),
                    }),
                );
            }
        }

        surface.rebuild();
        self.surface = surface;
        self.dispatcher = dispatcher;
        self.route_intents = route_intents;
    }
}

const fn drawer_header_route_id(surface_index: usize, item_index: usize) -> UiRouteId {
    UiRouteId::new(DRAWER_HEADER_ROUTE_ID_BASE + surface_index as u64 * 1_000 + item_index as u64)
}
