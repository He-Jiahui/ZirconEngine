use std::collections::BTreeMap;

use zircon_ui::{
    dispatch::UiPointerDispatcher, event_ui::UiTreeId, UiInputPolicy, UiSurface, UiTreeNode,
};

use super::base_state::base_state;
use super::constants::{
    ROOT_NODE_ID, STRIP_NODE_ID, STRIP_X, STRIP_Y, TAB_GAP, TAB_HEIGHT, TAB_MIN_WIDTH,
};
use super::register_handled_pointer_node::register_handled_pointer_node;
use super::root_frame::root_frame;
use super::tab_node_id::tab_node_id;
use super::workbench_host_page_pointer_bridge::WorkbenchHostPagePointerBridge;
use super::workbench_host_page_pointer_target::WorkbenchHostPagePointerTarget;

impl WorkbenchHostPagePointerBridge {
    pub(super) fn rebuild_surface(&mut self) {
        let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.host_page.pointer"));
        let mut dispatcher = UiPointerDispatcher::default();
        let mut targets = BTreeMap::new();

        surface.tree.insert_root(
            UiTreeNode::new(
                ROOT_NODE_ID,
                zircon_ui::event_ui::UiNodePath::new("editor.host_page.root"),
            )
            .with_frame(root_frame(&self.layout))
            .with_state_flags(base_state(false)),
        );
        surface
            .tree
            .insert_child(
                ROOT_NODE_ID,
                UiTreeNode::new(
                    STRIP_NODE_ID,
                    zircon_ui::event_ui::UiNodePath::new("editor.host_page.strip"),
                )
                .with_frame(self.layout.strip_frame)
                .with_z_index(10)
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(base_state(true)),
            )
            .expect("host page root must exist");

        let mut next_x = self.layout.strip_frame.x + STRIP_X;
        for (item_index, item) in self.layout.items.iter().enumerate() {
            let node_id = tab_node_id(item_index);
            let frame = self
                .measured_frames
                .get(item_index)
                .and_then(|frame| *frame)
                .unwrap_or(zircon_ui::UiFrame::new(
                    next_x,
                    self.layout.strip_frame.y + STRIP_Y,
                    TAB_MIN_WIDTH,
                    TAB_HEIGHT,
                ));
            next_x = frame.x + frame.width + TAB_GAP;
            surface
                .tree
                .insert_child(
                    STRIP_NODE_ID,
                    UiTreeNode::new(
                        node_id,
                        zircon_ui::event_ui::UiNodePath::new(format!(
                            "editor.host_page/tab_{item_index}"
                        )),
                    )
                    .with_frame(frame)
                    .with_z_index(20 + item_index as i32)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_state_flags(base_state(true)),
                )
                .expect("host page strip must exist");
            register_handled_pointer_node(&mut dispatcher, node_id);
            targets.insert(
                node_id,
                WorkbenchHostPagePointerTarget::Tab {
                    item_index,
                    page_id: item.page_id.clone(),
                },
            );
        }

        surface.rebuild();
        self.surface = surface;
        self.dispatcher = dispatcher;
        self.targets = targets;
    }
}
