use std::collections::BTreeMap;

use zircon_ui::{
    UiInputPolicy, UiNodeId, UiNodePath, UiPointerDispatcher, UiSurface, UiTreeId, UiTreeNode,
};

use super::base_state::base_state;
use super::constants::{
    ROOT_NODE_ID, STRIP_X, STRIP_Y, SURFACE_NODE_ID_BASE, TAB_GAP, TAB_HEIGHT, TAB_MIN_WIDTH,
    TAB_NODE_ID_BASE,
};
use super::register_handled_pointer_node::register_handled_pointer_node;
use super::root_frame::root_frame;
use super::workbench_drawer_header_pointer_bridge::WorkbenchDrawerHeaderPointerBridge;
use super::workbench_drawer_header_pointer_target::WorkbenchDrawerHeaderPointerTarget;

impl WorkbenchDrawerHeaderPointerBridge {
    pub(super) fn rebuild_surface(&mut self) {
        let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.drawer_header.pointer"));
        let mut dispatcher = UiPointerDispatcher::default();
        let mut targets = BTreeMap::new();

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

            let measured = self
                .measured_frames
                .get(surface_layout.key.as_str())
                .cloned()
                .unwrap_or_else(|| vec![None; surface_layout.items.len()]);
            let mut next_x = surface_layout.strip_frame.x + STRIP_X;

            for (item_index, item) in surface_layout.items.iter().enumerate() {
                let frame = measured
                    .get(item_index)
                    .and_then(|frame| *frame)
                    .unwrap_or_else(|| {
                        zircon_ui::UiFrame::new(
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
                targets.insert(
                    node_id,
                    WorkbenchDrawerHeaderPointerTarget::Tab {
                        surface_key: surface_layout.key.clone(),
                        item_index,
                        slot: item.slot.clone(),
                        instance_id: item.instance_id.clone(),
                    },
                );
            }
        }

        surface.rebuild();
        self.surface = surface;
        self.dispatcher = dispatcher;
        self.targets = targets;
    }
}
