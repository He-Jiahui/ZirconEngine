use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodePath, UiTreeId},
    tree::{UiInputPolicy, UiTreeNode},
};

use crate::ui::retained_host::route_intent::{EditorRouteIntent, EditorRouteIntentMap};

use super::base_state::base_state;
use super::constants::{OVERFLOW_NODE_ID, ROOT_NODE_ID, STRIP_NODE_ID};
use super::host_page_pointer_bridge::HostPagePointerBridge;
use super::host_page_pointer_route::HostPagePointerRoute;
use super::register_handled_pointer_node::register_handled_pointer_node;
use super::root_frame::root_frame;
use super::tab_node_id::{
    close_node_id, close_route_id, overflow_route_id, tab_node_id, tab_route_id,
};

impl HostPagePointerBridge {
    pub(super) fn rebuild_surface(&mut self) {
        let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.host_page.pointer"));
        let mut dispatcher = UiPointerDispatcher::default();
        let mut route_intents = EditorRouteIntentMap::default();

        surface.tree.insert_root(
            UiTreeNode::new(ROOT_NODE_ID, UiNodePath::new("editor.host_page.root"))
                .with_frame(root_frame(&self.layout))
                .with_state_flags(base_state(false)),
        );
        surface
            .tree
            .insert_child(
                ROOT_NODE_ID,
                UiTreeNode::new(STRIP_NODE_ID, UiNodePath::new("editor.host_page.strip"))
                    .with_frame(self.layout.strip_frame)
                    .with_z_index(10)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_state_flags(base_state(true)),
            )
            .expect("host page root must exist");

        for tab in &self.layout.tabs {
            let item_index = tab.page_index;
            let node_id = tab_node_id(item_index);
            surface
                .tree
                .insert_child(
                    STRIP_NODE_ID,
                    UiTreeNode::new(
                        node_id,
                        UiNodePath::new(format!("editor.host_page/tab_{item_index}")),
                    )
                    .with_frame(tab.frame)
                    .with_z_index(20 + item_index as i32)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_state_flags(base_state(true)),
                )
                .expect("host page strip must exist");
            register_handled_pointer_node(&mut dispatcher, node_id);
            route_intents.bind_node(
                node_id,
                tab_route_id(item_index),
                EditorRouteIntent::HostPage(HostPagePointerRoute::Tab {
                    item_index,
                    page_id: tab.page_id.clone(),
                }),
            );

            let close_target = self
                .layout
                .items
                .get(item_index)
                .and_then(|item| item.close_instance_id.as_ref());
            if let (Some(close_frame), Some(instance_id)) = (tab.close_frame, close_target) {
                let close_node_id = close_node_id(item_index);
                surface
                    .tree
                    .insert_child(
                        STRIP_NODE_ID,
                        UiTreeNode::new(
                            close_node_id,
                            UiNodePath::new(format!("editor.host_page/tab_{item_index}/close")),
                        )
                        .with_frame(close_frame)
                        .with_z_index(40 + item_index as i32)
                        .with_input_policy(UiInputPolicy::Receive)
                        .with_state_flags(base_state(true)),
                    )
                    .expect("host page strip must exist");
                register_handled_pointer_node(&mut dispatcher, close_node_id);
                route_intents.bind_node(
                    close_node_id,
                    close_route_id(item_index),
                    EditorRouteIntent::HostPage(HostPagePointerRoute::Close {
                        item_index,
                        instance_id: instance_id.clone(),
                    }),
                );
            }
        }

        if let Some(overflow) = &self.layout.overflow {
            surface
                .tree
                .insert_child(
                    STRIP_NODE_ID,
                    UiTreeNode::new(
                        OVERFLOW_NODE_ID,
                        UiNodePath::new("editor.host_page/overflow"),
                    )
                    .with_frame(overflow.frame)
                    .with_z_index(90)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_state_flags(base_state(true)),
                )
                .expect("host page strip must exist");
            register_handled_pointer_node(&mut dispatcher, OVERFLOW_NODE_ID);
            route_intents.bind_node(
                OVERFLOW_NODE_ID,
                overflow_route_id(),
                EditorRouteIntent::HostPage(HostPagePointerRoute::Overflow {
                    hidden_page_indices: overflow.hidden_page_indices.clone(),
                }),
            );
        }

        surface.rebuild();
        self.surface = surface;
        self.dispatcher = dispatcher;
        self.route_intents = route_intents;
    }
}
