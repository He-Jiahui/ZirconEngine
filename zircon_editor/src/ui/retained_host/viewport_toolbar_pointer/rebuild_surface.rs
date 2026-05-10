use std::collections::BTreeMap;

use zircon_runtime::ui::{
    dispatch::UiPointerDispatcher, surface::UiSurface, tree::UiRuntimeTreeAccessExt,
};
use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiNodePath, UiTreeId};
use zircon_runtime_interface::ui::tree::{UiInputPolicy, UiTreeNode};

use super::base_state::base_state;
use super::constants::{CONTROL_NODE_ID_BASE, ROOT_NODE_ID, SURFACE_NODE_ID_BASE};
use super::register_handled_pointer_node::register_handled_pointer_node;
use super::root_frame::root_frame;
use super::route_for_control::route_for_control;
use super::viewport_toolbar_pointer_bridge::ViewportToolbarPointerBridge;
use super::viewport_toolbar_pointer_target::ViewportToolbarPointerTarget;

impl ViewportToolbarPointerBridge {
    pub(super) fn rebuild_surface(&mut self) {
        let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.viewport_toolbar.pointer"));
        let mut dispatcher = UiPointerDispatcher::default();
        let mut targets = BTreeMap::new();

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

            let Some(active_control) = self.active_controls.get(surface_layout.key.as_str()) else {
                continue;
            };
            let route = route_for_control(surface_layout.key.as_str(), &active_control.action_key)
                .expect("active viewport toolbar action must stay valid");
            let control_node_id = UiNodeId::new(CONTROL_NODE_ID_BASE + surface_index as u64);
            surface
                .tree
                .insert_child(
                    surface_node_id,
                    UiTreeNode::new(
                        control_node_id,
                        UiNodePath::new(format!(
                            "editor.viewport_toolbar/{}/{}",
                            surface_layout.key, active_control.action_key
                        )),
                    )
                    .with_frame(active_control.frame)
                    .with_z_index(100 + surface_index as i32)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_state_flags(base_state(true)),
                )
                .expect("viewport toolbar surface must exist");
            register_handled_pointer_node(&mut dispatcher, control_node_id);
            targets.insert(control_node_id, ViewportToolbarPointerTarget { route });
        }

        surface.rebuild();
        self.surface = surface;
        self.dispatcher = dispatcher;
        self.targets = targets;
    }
}
