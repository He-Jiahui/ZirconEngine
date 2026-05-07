use std::collections::BTreeMap;

use zircon_runtime::ui::{
    dispatch::UiPointerDispatcher, surface::UiSurface, tree::UiRuntimeTreeAccessExt,
};
use zircon_runtime_interface::ui::{
    dispatch::UiPointerDispatchEffect,
    event_ui::{UiNodePath, UiTreeId},
    layout::{
        UiAxis, UiContainerKind, UiFrame, UiScrollState, UiScrollableBoxConfig,
        UiScrollbarVisibility,
    },
    surface::UiPointerEventKind,
    tree::{UiInputPolicy, UiTreeNode},
};

use super::constants::{DISMISS_NODE_ID, POPUP_ROW_HEIGHT, ROOT_NODE_ID};
use super::host_menu_pointer_bridge::HostMenuPointerBridge;
use super::host_menu_pointer_target::HostMenuPointerTarget;
use super::menu_item_spec::MenuItemSpec;
use super::menu_item_tree::menu_item_route_index;
use super::menu_items_for_layout::menu_items_for_layout;
use super::node_ids::{menu_button_node_id, menu_item_node_id, popup_node_id};
use super::popup_layout::{
    clipped_menu_button_frame, popup_grid_layout, popup_scroll_metrics, submenu_popup_grid_layout,
};
use super::register_handled_pointer_node::register_handled_pointer_node;
use super::state_flags::base_state;

impl HostMenuPointerBridge {
    pub(in crate::ui::slint_host::menu_pointer) fn rebuild_surface(&mut self) {
        let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.workbench.menu_pointer"));
        let mut dispatcher = UiPointerDispatcher::default();
        let mut targets = BTreeMap::new();

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
            targets.insert(node_id, HostMenuPointerTarget::MenuButton(menu_index));
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
                UiPointerDispatchEffect::handled()
            });
            dispatcher.register(DISMISS_NODE_ID, UiPointerEventKind::Down, |_context| {
                UiPointerDispatchEffect::handled()
            });
            targets.insert(DISMISS_NODE_ID, HostMenuPointerTarget::DismissOverlay);

            let root_items = menu_items_for_layout(&self.layout, menu_index);
            let root_grid = popup_grid_layout(
                &self.layout,
                menu_index,
                self.state.popup_scroll_offset,
                self.state.menu_bar_scroll_offset,
            );
            let mut visible_items = root_items.clone();
            let mut popup_path = Vec::new();
            let mut row_frames = insert_popup_layer(PopupLayerInsert {
                surface: &mut surface,
                dispatcher: &mut dispatcher,
                targets: &mut targets,
                menu_index,
                level: 0,
                popup_path: &popup_path,
                root_items: &root_items,
                items: &visible_items,
                grid: root_grid,
                root_scroll_metrics: Some(popup_scroll_metrics(&self.layout, menu_index)),
                root_scroll_offset: self.state.popup_scroll_offset,
            });

            for (level, selected_index) in self.state.open_submenu_path.iter().copied().enumerate()
            {
                let Some(branch_item) = visible_items.get(selected_index) else {
                    break;
                };
                if !branch_item.enabled || !branch_item.has_children() {
                    break;
                }
                let Some(anchor_frame) = row_frames.get(selected_index).copied() else {
                    break;
                };

                popup_path.push(selected_index);
                visible_items = branch_item.children.clone();
                let child_grid =
                    submenu_popup_grid_layout(&self.layout, anchor_frame, visible_items.len());
                row_frames = insert_popup_layer(PopupLayerInsert {
                    surface: &mut surface,
                    dispatcher: &mut dispatcher,
                    targets: &mut targets,
                    menu_index,
                    level: level + 1,
                    popup_path: &popup_path,
                    root_items: &root_items,
                    items: &visible_items,
                    grid: child_grid,
                    root_scroll_metrics: None,
                    root_scroll_offset: 0.0,
                });
            }
        }

        surface.rebuild();
        self.surface = surface;
        self.dispatcher = dispatcher;
        self.targets = targets;
    }
}

struct PopupLayerInsert<'a> {
    surface: &'a mut UiSurface,
    dispatcher: &'a mut UiPointerDispatcher,
    targets:
        &'a mut BTreeMap<zircon_runtime_interface::ui::event_ui::UiNodeId, HostMenuPointerTarget>,
    menu_index: usize,
    level: usize,
    popup_path: &'a [usize],
    root_items: &'a [MenuItemSpec],
    items: &'a [MenuItemSpec],
    grid: super::popup_layout::PopupGridLayout,
    root_scroll_metrics: Option<(f32, f32)>,
    root_scroll_offset: f32,
}

fn insert_popup_layer(args: PopupLayerInsert<'_>) -> Vec<UiFrame> {
    let popup_id = popup_node_id(args.level);
    let mut popup_node = UiTreeNode::new(
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

    if let Some((viewport_extent, content_extent)) = args.root_scroll_metrics {
        if content_extent > viewport_extent {
            popup_node = popup_node
                .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                    axis: UiAxis::Vertical,
                    gap: 0.0,
                    scrollbar_visibility: UiScrollbarVisibility::Auto,
                    virtualization: None,
                }))
                .with_scroll_state(UiScrollState {
                    offset: args.root_scroll_offset,
                    viewport_extent,
                    content_extent,
                });
        }
    }

    args.surface
        .tree
        .insert_child(ROOT_NODE_ID, popup_node)
        .expect("menu pointer root must exist");
    register_handled_pointer_node(args.dispatcher, popup_id);
    args.targets.insert(
        popup_id,
        HostMenuPointerTarget::PopupSurface(args.menu_index),
    );

    let mut row_frames = Vec::with_capacity(args.items.len());
    for (visible_index, item) in args.items.iter().enumerate() {
        let node_id = menu_item_node_id(
            args.level,
            menu_item_route_index(args.root_items, &item_path(args.popup_path, visible_index))
                .unwrap_or(visible_index),
        );
        let frame = popup_item_frame(args.grid, visible_index);
        row_frames.push(frame);

        let mut path = args.popup_path.to_vec();
        path.push(visible_index);
        let item_index = menu_item_route_index(args.root_items, &path).unwrap_or(visible_index);
        let is_branch = item.enabled && item.has_children();
        let is_leaf = item.enabled && item.action_id.is_some();
        let interactive = is_branch || is_leaf;

        args.surface
            .tree
            .insert_child(
                popup_id,
                UiTreeNode::new(
                    node_id,
                    UiNodePath::new(format!(
                        "editor.workbench.menu/popup_{}/level_{}/item_{}",
                        args.menu_index, args.level, item_index
                    )),
                )
                .with_frame(frame)
                .with_z_index(110 + args.level as i32 * 40 + visible_index as i32)
                .with_input_policy(if interactive {
                    UiInputPolicy::Receive
                } else {
                    UiInputPolicy::Ignore
                })
                .with_state_flags(base_state(interactive)),
            )
            .expect("popup node must exist");

        if is_branch {
            register_handled_pointer_node(args.dispatcher, node_id);
            args.targets.insert(
                node_id,
                HostMenuPointerTarget::SubmenuBranch {
                    menu_index: args.menu_index,
                    item_index,
                    item_path: path,
                },
            );
        } else if is_leaf {
            register_handled_pointer_node(args.dispatcher, node_id);
            args.targets.insert(
                node_id,
                HostMenuPointerTarget::MenuItem {
                    menu_index: args.menu_index,
                    item_index,
                    item_path: path,
                    action_id: item
                        .action_id
                        .clone()
                        .expect("interactive leaf items need an action"),
                },
            );
        }
    }
    row_frames
}

fn item_path(parent_path: &[usize], visible_index: usize) -> Vec<usize> {
    let mut path = parent_path.to_vec();
    path.push(visible_index);
    path
}

fn popup_item_frame(grid: super::popup_layout::PopupGridLayout, item_index: usize) -> UiFrame {
    let column = if grid.rows_per_column == 0 {
        0
    } else {
        item_index / grid.rows_per_column
    };
    let row = if grid.rows_per_column == 0 {
        0
    } else {
        item_index % grid.rows_per_column
    };
    UiFrame::new(
        grid.content_frame.x + column as f32 * grid.column_width,
        grid.content_frame.y + row as f32 * grid.row_step - grid.scroll_offset,
        grid.column_width,
        POPUP_ROW_HEIGHT,
    )
}
