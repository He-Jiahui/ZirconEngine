use std::collections::BTreeMap;

use zircon_runtime::ui::{
    dispatch::UiPointerDispatcher, event_ui::UiTreeId, layout::UiAxis,
    layout::UiContainerKind, layout::UiFrame, layout::UiScrollState,
    layout::UiScrollableBoxConfig, layout::UiScrollbarVisibility, surface::UiSurface,
    tree::UiInputPolicy, tree::UiTreeNode,
};

use super::base_state::base_state;
use super::constants::{
    ROOT_NODE_ID, ROW_GAP, ROW_HEIGHT, ROW_WIDTH_INSET, ROW_X, ROW_Y, VIEWPORT_NODE_ID,
};
use super::content_height::content_height;
use super::hierarchy_pointer_bridge::HierarchyPointerBridge;
use super::hierarchy_pointer_target::HierarchyPointerTarget;
use super::item_node_id::item_node_id;
use super::register_handled_pointer_node::register_handled_pointer_node;
use super::viewport_frame::viewport_frame;

impl HierarchyPointerBridge {
    pub(super) fn rebuild_surface(&mut self) {
        let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.hierarchy.pointer"));
        let mut dispatcher = UiPointerDispatcher::default();
        let mut targets = BTreeMap::new();

        surface.tree.insert_root(
            UiTreeNode::new(
                ROOT_NODE_ID,
                zircon_runtime::ui::event_ui::UiNodePath::new("editor.hierarchy.root"),
            )
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
                    zircon_runtime::ui::event_ui::UiNodePath::new("editor.hierarchy.viewport"),
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
                    content_extent: content_height(self.layout.node_ids.len()),
                })
                .with_state_flags(base_state(true)),
            )
            .expect("hierarchy root must exist");
        register_handled_pointer_node(&mut dispatcher, VIEWPORT_NODE_ID);
        targets.insert(VIEWPORT_NODE_ID, HierarchyPointerTarget::ListSurface);

        let row_width = (self.layout.pane_width - ROW_WIDTH_INSET).max(0.0);
        for (item_index, node_id) in self.layout.node_ids.iter().enumerate() {
            let item_node_id = item_node_id(item_index);
            surface
                .tree
                .insert_child(
                    VIEWPORT_NODE_ID,
                    UiTreeNode::new(
                        item_node_id,
                        zircon_runtime::ui::event_ui::UiNodePath::new(format!(
                            "editor.hierarchy/item_{item_index}"
                        )),
                    )
                    .with_frame(UiFrame::new(
                        ROW_X,
                        ROW_Y + item_index as f32 * (ROW_HEIGHT + ROW_GAP)
                            - self.state.scroll_offset,
                        row_width,
                        ROW_HEIGHT,
                    ))
                    .with_z_index(20 + item_index as i32)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_state_flags(base_state(true)),
                )
                .expect("hierarchy viewport must exist");
            register_handled_pointer_node(&mut dispatcher, item_node_id);
            targets.insert(
                item_node_id,
                HierarchyPointerTarget::Node {
                    item_index,
                    node_id: node_id.clone(),
                },
            );
        }

        surface.rebuild();
        self.surface = surface;
        self.dispatcher = dispatcher;
        self.targets = targets;
    }
}
