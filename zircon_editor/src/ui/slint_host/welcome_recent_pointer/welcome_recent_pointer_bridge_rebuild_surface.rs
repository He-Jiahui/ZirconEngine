use std::collections::BTreeMap;

use zircon_ui::{
    UiAxis, UiContainerKind, UiFrame, UiInputPolicy, UiNodePath, UiPointerDispatcher,
    UiScrollState, UiScrollableBoxConfig, UiScrollbarVisibility, UiSurface, UiTreeId, UiTreeNode,
};

use super::constants::{
    BUTTON_Y, ITEM_GAP, ITEM_HEIGHT, OPEN_BUTTON_X, REMOVE_BUTTON_X, ROOT_NODE_ID,
    VIEWPORT_INNER_WIDTH_INSET, VIEWPORT_NODE_ID,
};
use super::helper::{
    base_state, button_size, content_height, item_node_id, open_button_node_id,
    remove_button_node_id, viewport_frame,
};
use super::register_handled_pointer_node::register_handled_pointer_node;
use super::welcome_recent_pointer_action::WelcomeRecentPointerAction;
use super::welcome_recent_pointer_bridge::WelcomeRecentPointerBridge;
use super::welcome_recent_pointer_target::WelcomeRecentPointerTarget;

impl WelcomeRecentPointerBridge {
    pub(in crate::ui::slint_host::welcome_recent_pointer) fn rebuild_surface(&mut self) {
        let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.welcome.recent_pointer"));
        let mut dispatcher = UiPointerDispatcher::default();
        let mut targets = BTreeMap::new();

        surface.tree.insert_root(
            UiTreeNode::new(ROOT_NODE_ID, UiNodePath::new("editor.welcome.recent.root"))
                .with_frame(UiFrame::new(
                    0.0,
                    0.0,
                    self.layout.pane_size.width.max(0.0),
                    self.layout.pane_size.height.max(0.0),
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
                    UiNodePath::new("editor.welcome.recent.viewport"),
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
                    content_extent: content_height(self.layout.recent_project_paths.len()),
                })
                .with_state_flags(base_state(true)),
            )
            .expect("welcome recent pointer root must exist");
        register_handled_pointer_node(&mut dispatcher, VIEWPORT_NODE_ID);
        targets.insert(VIEWPORT_NODE_ID, WelcomeRecentPointerTarget::ListSurface);

        let row_width = (viewport.width - VIEWPORT_INNER_WIDTH_INSET).max(0.0);
        let (button_width, button_height) = button_size();
        for (item_index, path) in self.layout.recent_project_paths.iter().enumerate() {
            let item_node_id = item_node_id(item_index);
            let item_frame = UiFrame::new(
                viewport.x,
                viewport.y + item_index as f32 * (ITEM_HEIGHT + ITEM_GAP)
                    - self.state.scroll_offset,
                row_width,
                ITEM_HEIGHT,
            );
            surface
                .tree
                .insert_child(
                    VIEWPORT_NODE_ID,
                    UiTreeNode::new(
                        item_node_id,
                        UiNodePath::new(format!("editor.welcome.recent/item_{item_index}")),
                    )
                    .with_frame(item_frame)
                    .with_z_index(20 + item_index as i32)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_state_flags(base_state(true)),
                )
                .expect("welcome viewport must exist");
            register_handled_pointer_node(&mut dispatcher, item_node_id);
            targets.insert(item_node_id, WelcomeRecentPointerTarget::Item(item_index));

            let open_node_id = open_button_node_id(item_index);
            surface
                .tree
                .insert_child(
                    item_node_id,
                    UiTreeNode::new(
                        open_node_id,
                        UiNodePath::new(format!("editor.welcome.recent/item_{item_index}/open")),
                    )
                    .with_frame(UiFrame::new(
                        item_frame.x + OPEN_BUTTON_X,
                        item_frame.y + BUTTON_Y,
                        button_width,
                        button_height,
                    ))
                    .with_z_index(120 + item_index as i32)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_state_flags(base_state(true)),
                )
                .expect("welcome recent item must exist");
            register_handled_pointer_node(&mut dispatcher, open_node_id);
            targets.insert(
                open_node_id,
                WelcomeRecentPointerTarget::Action {
                    item_index,
                    action: WelcomeRecentPointerAction::Open,
                    path: path.clone(),
                },
            );

            let remove_node_id = remove_button_node_id(item_index);
            surface
                .tree
                .insert_child(
                    item_node_id,
                    UiTreeNode::new(
                        remove_node_id,
                        UiNodePath::new(format!("editor.welcome.recent/item_{item_index}/remove")),
                    )
                    .with_frame(UiFrame::new(
                        item_frame.x + REMOVE_BUTTON_X,
                        item_frame.y + BUTTON_Y,
                        button_width,
                        button_height,
                    ))
                    .with_z_index(220 + item_index as i32)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_state_flags(base_state(true)),
                )
                .expect("welcome recent item must exist");
            register_handled_pointer_node(&mut dispatcher, remove_node_id);
            targets.insert(
                remove_node_id,
                WelcomeRecentPointerTarget::Action {
                    item_index,
                    action: WelcomeRecentPointerAction::Remove,
                    path: path.clone(),
                },
            );
        }

        surface.rebuild();
        self.surface = surface;
        self.dispatcher = dispatcher;
        self.targets = targets;
    }
}
