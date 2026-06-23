use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};
use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiNodePath, UiRouteId, UiTreeId};
use zircon_runtime_interface::ui::tree::{UiInputPolicy, UiTreeNode};

use crate::ui::retained_host::route_intent::{EditorRouteIntent, EditorRouteIntentMap};

use super::base_state::base_state;
use super::constants::{
    CONTROL_NODE_ID_BASE, ROOT_NODE_ID, SURFACE_NODE_ID_BASE, VIEWPORT_TOOLBAR_ROUTE_ID_BASE,
    VIEWPORT_TOOLBAR_SURFACE_STRIDE,
};
use super::register_handled_pointer_node::register_handled_pointer_node;
use super::root_frame::root_frame;
use super::route_for_control::route_for_control;
use super::viewport_toolbar_pointer_bridge::ViewportToolbarPointerBridge;

impl ViewportToolbarPointerBridge {
    pub(super) fn rebuild_surface(&mut self) {
        let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.viewport_toolbar.pointer"));
        let mut dispatcher = UiPointerDispatcher::default();
        let mut route_intents = EditorRouteIntentMap::default();

        surface.tree.insert_root(
            UiTreeNode::new(
                ROOT_NODE_ID,
                UiNodePath::new("editor.viewport_toolbar.root"),
            )
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
                        UiNodePath::new(format!("editor.viewport_toolbar/{}", surface_layout.key)),
                    )
                    .with_frame(surface_layout.frame)
                    .with_z_index(10 + surface_index as i32)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_state_flags(base_state(true)),
                )
                .expect("viewport toolbar root must exist");

            let Some(controls) = self.controls_by_surface.get(surface_layout.key.as_str()) else {
                continue;
            };
            for (control_index, control) in controls.iter().enumerate() {
                let route = route_for_control(surface_layout.key.as_str(), &control.action_key)
                    .expect("viewport toolbar action must stay valid");
                let control_node_id =
                    viewport_toolbar_control_node_id(surface_index, control_index);
                surface
                    .tree
                    .insert_child(
                        surface_node_id,
                        UiTreeNode::new(
                            control_node_id,
                            UiNodePath::new(format!(
                                "editor.viewport_toolbar/{}/{}",
                                surface_layout.key, control.action_key
                            )),
                        )
                        .with_frame(control.frame)
                        .with_z_index(100 + control_index as i32)
                        .with_input_policy(UiInputPolicy::Receive)
                        .with_state_flags(base_state(true)),
                    )
                    .expect("viewport toolbar surface must exist");
                register_handled_pointer_node(&mut dispatcher, control_node_id);
                route_intents.bind_node(
                    control_node_id,
                    viewport_toolbar_route_id(surface_index, control_index),
                    EditorRouteIntent::ViewportToolbar(route),
                );
            }
        }

        surface.rebuild();
        self.surface = surface;
        self.dispatcher = dispatcher;
        self.route_intents = route_intents;
    }
}

const fn viewport_toolbar_control_node_id(surface_index: usize, control_index: usize) -> UiNodeId {
    UiNodeId::new(
        CONTROL_NODE_ID_BASE
            + surface_index as u64 * VIEWPORT_TOOLBAR_SURFACE_STRIDE
            + control_index as u64,
    )
}

const fn viewport_toolbar_route_id(surface_index: usize, control_index: usize) -> UiRouteId {
    UiRouteId::new(
        VIEWPORT_TOOLBAR_ROUTE_ID_BASE
            + surface_index as u64 * VIEWPORT_TOOLBAR_SURFACE_STRIDE
            + control_index as u64,
    )
}
