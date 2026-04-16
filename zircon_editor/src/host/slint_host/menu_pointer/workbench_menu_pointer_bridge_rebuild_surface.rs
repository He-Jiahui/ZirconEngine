use std::collections::BTreeMap;

use zircon_ui::{
    UiAxis, UiContainerKind, UiFrame, UiInputPolicy, UiNodePath, UiPointerDispatcher,
    UiPointerEventKind, UiScrollState, UiScrollableBoxConfig, UiScrollbarVisibility, UiSurface,
    UiTreeId, UiTreeNode,
};

use super::constants::{
    DISMISS_NODE_ID, POPUP_NODE_ID, POPUP_ROW_GAP, POPUP_ROW_HEIGHT, ROOT_NODE_ID,
    WINDOW_MENU_INDEX,
};
use super::menu_items_for_layout::menu_items_for_layout;
use super::node_ids::{menu_button_node_id, menu_item_node_id};
use super::popup_layout::{popup_content_frame, popup_frame, popup_scroll_metrics};
use super::register_handled_pointer_node::register_handled_pointer_node;
use super::state_flags::base_state;
use super::workbench_menu_pointer_bridge::WorkbenchMenuPointerBridge;
use super::workbench_menu_pointer_target::WorkbenchMenuPointerTarget;

impl WorkbenchMenuPointerBridge {
    pub(in crate::host::slint_host::menu_pointer) fn rebuild_surface(&mut self) {
        let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.workbench.menu_pointer"));
        let mut dispatcher = UiPointerDispatcher::default();
        let mut targets = BTreeMap::new();

        surface.tree.insert_root(
            UiTreeNode::new(ROOT_NODE_ID, UiNodePath::new("editor.workbench.menu.root"))
                .with_frame(self.layout.shell_frame)
                .with_state_flags(base_state(false)),
        );

        for menu_index in 0..self.layout.button_frames.len() {
            let node_id = menu_button_node_id(menu_index);
            surface
                .tree
                .insert_child(
                    ROOT_NODE_ID,
                    UiTreeNode::new(
                        node_id,
                        UiNodePath::new(format!("editor.workbench.menu/button_{menu_index}")),
                    )
                    .with_frame(self.layout.button_frames[menu_index])
                    .with_z_index(200 + menu_index as i32)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_state_flags(base_state(true)),
                )
                .expect("menu pointer root must exist");
            register_handled_pointer_node(&mut dispatcher, node_id);
            targets.insert(node_id, WorkbenchMenuPointerTarget::MenuButton(menu_index));
        }

        if let Some(menu_index) = self.state.open_menu_index {
            surface
                .tree
                .insert_child(
                    ROOT_NODE_ID,
                    UiTreeNode::new(
                        DISMISS_NODE_ID,
                        UiNodePath::new("editor.workbench.menu/dismiss"),
                    )
                    .with_frame(self.layout.shell_frame)
                    .with_z_index(10)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_state_flags(base_state(true)),
                )
                .expect("menu pointer root must exist");
            dispatcher.register(DISMISS_NODE_ID, UiPointerEventKind::Move, |_context| {
                zircon_ui::UiPointerDispatchEffect::handled()
            });
            dispatcher.register(DISMISS_NODE_ID, UiPointerEventKind::Down, |_context| {
                zircon_ui::UiPointerDispatchEffect::handled()
            });
            targets.insert(DISMISS_NODE_ID, WorkbenchMenuPointerTarget::DismissOverlay);

            let popup_frame = popup_frame(&self.layout, menu_index);
            let popup_node = {
                let node = UiTreeNode::new(
                    POPUP_NODE_ID,
                    UiNodePath::new(format!("editor.workbench.menu/popup_{menu_index}")),
                )
                .with_frame(popup_frame)
                .with_z_index(100)
                .with_input_policy(UiInputPolicy::Receive)
                .with_clip_to_bounds(true)
                .with_state_flags(base_state(true));
                if menu_index == WINDOW_MENU_INDEX {
                    let (viewport_extent, content_extent) =
                        popup_scroll_metrics(&self.layout, menu_index);
                    node.with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                        axis: UiAxis::Vertical,
                        gap: 0.0,
                        scrollbar_visibility: UiScrollbarVisibility::Auto,
                        virtualization: None,
                    }))
                    .with_scroll_state(UiScrollState {
                        offset: self.state.popup_scroll_offset,
                        viewport_extent,
                        content_extent,
                    })
                } else {
                    node
                }
            };
            surface
                .tree
                .insert_child(ROOT_NODE_ID, popup_node)
                .expect("menu pointer root must exist");
            register_handled_pointer_node(&mut dispatcher, POPUP_NODE_ID);
            targets.insert(
                POPUP_NODE_ID,
                WorkbenchMenuPointerTarget::PopupSurface(menu_index),
            );

            let content_frame = popup_content_frame(popup_frame);
            for (item_index, item) in menu_items_for_layout(&self.layout, menu_index)
                .into_iter()
                .enumerate()
            {
                let node_id = menu_item_node_id(item_index);
                let frame = UiFrame::new(
                    content_frame.x,
                    content_frame.y + item_index as f32 * (POPUP_ROW_HEIGHT + POPUP_ROW_GAP)
                        - self.state.popup_scroll_offset,
                    content_frame.width,
                    POPUP_ROW_HEIGHT,
                );
                let interactive = item.enabled && item.action_id.is_some();
                surface
                    .tree
                    .insert_child(
                        POPUP_NODE_ID,
                        UiTreeNode::new(
                            node_id,
                            UiNodePath::new(format!(
                                "editor.workbench.menu/popup_{menu_index}/item_{item_index}"
                            )),
                        )
                        .with_frame(frame)
                        .with_z_index(110 + item_index as i32)
                        .with_input_policy(if interactive {
                            UiInputPolicy::Receive
                        } else {
                            UiInputPolicy::Ignore
                        })
                        .with_state_flags(base_state(interactive)),
                    )
                    .expect("popup node must exist");
                if interactive {
                    register_handled_pointer_node(&mut dispatcher, node_id);
                    targets.insert(
                        node_id,
                        WorkbenchMenuPointerTarget::MenuItem {
                            menu_index,
                            item_index,
                            action_id: item.action_id.expect("interactive items need an action"),
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
