use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodePath, UiTreeId},
    layout::UiFrame,
    tree::{UiInputPolicy, UiTreeNode},
};

use super::constants::{DISMISS_NODE_ID, ROOT_NODE_ID};
use super::host_menu_pointer_bridge::HostMenuPointerBridge;
use super::host_menu_pointer_route_intent::HostMenuPointerRouteIntent;
use super::node_ids::{
    dismiss_route_id, menu_button_node_id, menu_button_route_id, popup_node_id, popup_route_id,
};
use super::popup_layout::{
    clipped_menu_button_frame, popup_grid_layout, popup_item_frame, submenu_popup_grid_layout,
};
use super::register_handled_pointer_node::register_handled_pointer_node;
use super::state_flags::base_state;
use crate::ui::retained_host::route_intent::{EditorRouteIntent, EditorRouteIntentMap};

impl HostMenuPointerBridge {
    pub(in crate::ui::retained_host::menu_pointer) fn rebuild_surface(&mut self) {
        let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.workbench.menu_pointer"));
        let mut dispatcher = UiPointerDispatcher::default();
        let mut route_intents = EditorRouteIntentMap::default();

        surface.tree.insert_root(
            UiTreeNode::new(ROOT_NODE_ID, UiNodePath::new("editor.workbench.menu.root"))
                .with_frame(self.layout.shell_frame)
                .with_state_flags(base_state(false)),
        );

        for menu_index in 0..self.layout.button_frames.len() {
            let Some(button_frame) = clipped_menu_button_frame(
                &self.layout,
                menu_index,
                self.state.menu_bar_scroll_offset,
            ) else {
                continue;
            };
            let node_id = menu_button_node_id(menu_index);
            surface
                .tree
                .insert_child(
                    ROOT_NODE_ID,
                    UiTreeNode::new(
                        node_id,
                        UiNodePath::new(format!("editor.workbench.menu/button_{menu_index}")),
                    )
                    .with_frame(button_frame)
                    .with_z_index(200 + menu_index as i32)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_state_flags(base_state(true)),
                )
                .expect("menu pointer root must exist");
            register_handled_pointer_node(&mut dispatcher, node_id);
            route_intents.bind_node(
                node_id,
                menu_button_route_id(menu_index),
                EditorRouteIntent::Menu(HostMenuPointerRouteIntent::MenuButton(menu_index)),
            );
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
            register_handled_pointer_node(&mut dispatcher, DISMISS_NODE_ID);
            route_intents.bind_node(
                DISMISS_NODE_ID,
                dismiss_route_id(),
                EditorRouteIntent::Menu(HostMenuPointerRouteIntent::DismissOverlay),
            );

            let root_items = self.popup_items.as_slice();
            let mut grid = popup_grid_layout(
                &self.layout,
                menu_index,
                root_items.len(),
                self.state.popup_scroll_offset,
                self.state.menu_bar_scroll_offset,
            );
            let mut visible_items = root_items;
            insert_popup_layer(PopupLayerInsert {
                surface: &mut surface,
                dispatcher: &mut dispatcher,
                route_intents: &mut route_intents,
                menu_index,
                level: 0,
                grid,
            });

            for (level, selected_index) in self.state.open_submenu_path.iter().copied().enumerate()
            {
                let Some(branch_item) = visible_items.get(selected_index) else {
                    break;
                };
                if !branch_item.enabled || !branch_item.has_children() {
                    break;
                }
                let anchor_frame = popup_item_frame(grid, selected_index);
                visible_items = branch_item.children.as_slice();
                grid = submenu_popup_grid_layout(&self.layout, anchor_frame, visible_items.len());
                insert_popup_layer(PopupLayerInsert {
                    surface: &mut surface,
                    dispatcher: &mut dispatcher,
                    route_intents: &mut route_intents,
                    menu_index,
                    level: level + 1,
                    grid,
                });
            }
        }

        surface.rebuild();
        self.surface = surface;
        self.dispatcher = dispatcher;
        self.route_intents = route_intents;
        #[cfg(test)]
        {
            self.surface_authority_generation = self.surface_authority_generation.saturating_add(1);
        }
    }
}

struct PopupLayerInsert<'a> {
    surface: &'a mut UiSurface,
    dispatcher: &'a mut UiPointerDispatcher,
    route_intents: &'a mut EditorRouteIntentMap,
    menu_index: usize,
    level: usize,
    grid: super::popup_layout::PopupGridLayout,
}

fn insert_popup_layer(args: PopupLayerInsert<'_>) {
    let popup_id = popup_node_id(args.level);
    let popup_node = UiTreeNode::new(
        popup_id,
        UiNodePath::new(format!(
            "editor.workbench.menu/popup_{}/level_{}",
            args.menu_index, args.level
        )),
    )
    .with_frame(args.grid.frame)
    .with_z_index(100 + args.level as i32 * 40)
    .with_input_policy(UiInputPolicy::Receive)
    .with_clip_to_bounds(true)
    .with_state_flags(base_state(true));

    args.surface
        .tree
        .insert_child(ROOT_NODE_ID, popup_node)
        .expect("menu pointer root must exist");
    register_handled_pointer_node(args.dispatcher, popup_id);
    args.route_intents.bind_node(
        popup_id,
        popup_route_id(args.level),
        EditorRouteIntent::Menu(HostMenuPointerRouteIntent::PopupSurface(args.menu_index)),
    );
}
