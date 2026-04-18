use std::collections::BTreeMap;

use zircon_ui::{
    UiInputPolicy, UiNodeId, UiNodePath, UiPointerDispatcher, UiSurface, UiTreeId, UiTreeNode,
};

use super::constants::{
    CLOSE_EXTENT, CLOSE_X_OFFSET, CLOSE_Y_OFFSET, ROOT_NODE_ID, STRIP_X, STRIP_Y,
    SURFACE_NODE_ID_BASE, TAB_GAP, TAB_HEIGHT,
};
use super::helper::{base_state, close_node_id, root_frame, tab_min_width, tab_node_id};
use super::register_handled_pointer_node::register_handled_pointer_node;
use super::workbench_document_tab_pointer_bridge::WorkbenchDocumentTabPointerBridge;
use super::workbench_document_tab_pointer_target::WorkbenchDocumentTabPointerTarget;

impl WorkbenchDocumentTabPointerBridge {
    pub(in crate::ui::slint_host::document_tab_pointer) fn rebuild_surface(&mut self) {
        let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.document_tab.pointer"));
        let mut dispatcher = UiPointerDispatcher::default();
        let mut targets = BTreeMap::new();

        surface.tree.insert_root(
            UiTreeNode::new(ROOT_NODE_ID, UiNodePath::new("editor.document_tab.root"))
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
                        UiNodePath::new(format!("editor.document_tab/{}", surface_layout.key)),
                    )
                    .with_frame(surface_layout.strip_frame)
                    .with_z_index(10 + surface_index as i32)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_state_flags(base_state(true)),
                )
                .expect("document tab root must exist");

            let mut next_x = surface_layout.strip_frame.x + STRIP_X;
            let measured = self
                .measured_frames
                .get(surface_layout.key.as_str())
                .cloned()
                .unwrap_or_else(|| vec![None; surface_layout.items.len()]);
            for (item_index, item) in surface_layout.items.iter().enumerate() {
                let frame = measured
                    .get(item_index)
                    .and_then(|frame| *frame)
                    .unwrap_or_else(|| {
                        zircon_ui::UiFrame::new(
                            next_x,
                            surface_layout.strip_frame.y + STRIP_Y,
                            tab_min_width(surface_layout, item_index),
                            TAB_HEIGHT,
                        )
                    });
                next_x = frame.x + frame.width + TAB_GAP;
                let tab_node_id = tab_node_id(surface_index, item_index);
                surface
                    .tree
                    .insert_child(
                        surface_node_id,
                        UiTreeNode::new(
                            tab_node_id,
                            UiNodePath::new(format!(
                                "editor.document_tab/{}/tab_{item_index}",
                                surface_layout.key
                            )),
                        )
                        .with_frame(frame)
                        .with_z_index(20 + item_index as i32)
                        .with_input_policy(UiInputPolicy::Receive)
                        .with_state_flags(base_state(true)),
                    )
                    .expect("document tab surface must exist");
                register_handled_pointer_node(&mut dispatcher, tab_node_id);
                targets.insert(
                    tab_node_id,
                    WorkbenchDocumentTabPointerTarget::ActivateTab {
                        surface_key: surface_layout.key.clone(),
                        item_index,
                        instance_id: item.instance_id.clone(),
                    },
                );

                if item.closeable {
                    let close_node_id = close_node_id(surface_index, item_index);
                    surface
                        .tree
                        .insert_child(
                            surface_node_id,
                            UiTreeNode::new(
                                close_node_id,
                                UiNodePath::new(format!(
                                    "editor.document_tab/{}/tab_{item_index}/close",
                                    surface_layout.key
                                )),
                            )
                            .with_frame(zircon_ui::UiFrame::new(
                                frame.x + frame.width - CLOSE_X_OFFSET,
                                frame.y + CLOSE_Y_OFFSET,
                                CLOSE_EXTENT,
                                CLOSE_EXTENT,
                            ))
                            .with_z_index(40 + item_index as i32)
                            .with_input_policy(UiInputPolicy::Receive)
                            .with_state_flags(base_state(true)),
                        )
                        .expect("document tab surface must exist");
                    register_handled_pointer_node(&mut dispatcher, close_node_id);
                    targets.insert(
                        close_node_id,
                        WorkbenchDocumentTabPointerTarget::CloseTab {
                            surface_key: surface_layout.key.clone(),
                            item_index,
                            instance_id: item.instance_id.clone(),
                        },
                    );
                }
            }
        }

        surface.rebuild();
        self.surface = surface;
        self.dispatcher = dispatcher;
        self.targets = targets;
    }
}
