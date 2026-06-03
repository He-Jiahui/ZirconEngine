use winit::keyboard::{Key, NamedKey};

use super::data::{FrameRect, HostPaneInteractionStateData, TemplatePaneNodeData};
use super::frame_geometry::union_optional_frames;
use super::globals::PaneSurfaceHostContext;
use super::redraw::NativePointerDispatchResult;
use super::template_geometry::{frame_from_template_node, template_popup_bounds};
use super::template_popup_layout::{
    dropdown_option_popup_frame_within, dropdown_option_row_frame_within, menu_item_row_frame,
};
use super::window::UiHostWindow;
use crate::ui::retained_host::primitives::{ModelRc, SharedString};
use crate::ui::retained_host::workbench_popup_actions::WORKBENCH_POPUP_CANCEL_ACTION_ID;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WorkbenchPopupKeyboardCommand {
    Next,
    Previous,
    Accept,
    Cancel,
}

pub(super) fn workbench_popup_keyboard_command(key: &Key) -> Option<WorkbenchPopupKeyboardCommand> {
    match key {
        Key::Named(NamedKey::ArrowDown) => Some(WorkbenchPopupKeyboardCommand::Next),
        Key::Named(NamedKey::ArrowUp) => Some(WorkbenchPopupKeyboardCommand::Previous),
        Key::Named(NamedKey::Enter) => Some(WorkbenchPopupKeyboardCommand::Accept),
        Key::Named(NamedKey::Escape) => Some(WorkbenchPopupKeyboardCommand::Cancel),
        _ => None,
    }
}

pub(super) fn dispatch_workbench_popup_keyboard_command(
    ui: &UiHostWindow,
    command: WorkbenchPopupKeyboardCommand,
) -> NativePointerDispatchResult {
    let presentation = ui.get_host_presentation();
    let interaction = ui.get_pane_interaction_state();
    let bounds = template_popup_bounds(
        &presentation.host_shell.native_window_bounds,
        &presentation.workbench_window_nodes,
    );
    let Some(target) =
        active_popup_keyboard_target(&presentation.workbench_window_nodes, &interaction, &bounds)
    else {
        return NativePointerDispatchResult::idle();
    };

    match command {
        WorkbenchPopupKeyboardCommand::Next | WorkbenchPopupKeyboardCommand::Previous => {
            let Some(next) = target.next_row(command) else {
                return NativePointerDispatchResult::idle();
            };
            ui.set_hovered_template_row_for_pointer_move(
                target.control_id.clone(),
                target.dispatch_kind,
                next.action_id,
                next.value_text,
                next.frame.clone(),
            );
            NativePointerDispatchResult::region(
                union_optional_frames(Some(target.current_frame), Some(next.frame))
                    .unwrap_or_default(),
            )
        }
        WorkbenchPopupKeyboardCommand::Accept => {
            let Some(row) = target.current_row else {
                return NativePointerDispatchResult::idle();
            };
            let popup_frame = target.popup_frame;
            let pane_host = ui.global::<PaneSurfaceHostContext>();
            match target.dispatch_kind.as_str() {
                "workbench_option" => pane_host.invoke_component_showcase_option_selected(
                    target.control_id,
                    row.action_id,
                    row.value_text,
                ),
                "workbench_menu_item" => {
                    pane_host.invoke_surface_control_clicked(target.control_id, row.action_id)
                }
                _ => return NativePointerDispatchResult::idle(),
            }
            NativePointerDispatchResult::region_with_frame_update(popup_frame)
        }
        WorkbenchPopupKeyboardCommand::Cancel => {
            let popup_frame = target.popup_frame;
            let pane_host = ui.global::<PaneSurfaceHostContext>();
            pane_host.invoke_surface_control_clicked(
                target.control_id,
                WORKBENCH_POPUP_CANCEL_ACTION_ID.into(),
            );
            ui.clear_hovered_template_node_for_pointer_move();
            NativePointerDispatchResult::region_with_frame_update(popup_frame)
        }
    }
}

struct PopupKeyboardTarget {
    control_id: SharedString,
    dispatch_kind: SharedString,
    rows: Vec<PopupKeyboardRow>,
    current_index: usize,
    current_row: Option<PopupKeyboardRow>,
    current_frame: FrameRect,
    popup_frame: FrameRect,
}

impl PopupKeyboardTarget {
    fn next_row(&self, command: WorkbenchPopupKeyboardCommand) -> Option<PopupKeyboardRow> {
        if self.rows.is_empty() {
            return None;
        }
        let next_index = match command {
            WorkbenchPopupKeyboardCommand::Next => (self.current_index + 1) % self.rows.len(),
            WorkbenchPopupKeyboardCommand::Previous => {
                (self.current_index + self.rows.len() - 1) % self.rows.len()
            }
            WorkbenchPopupKeyboardCommand::Accept | WorkbenchPopupKeyboardCommand::Cancel => {
                self.current_index
            }
        };
        self.rows.get(next_index).cloned()
    }
}

#[derive(Clone)]
struct PopupKeyboardRow {
    action_id: SharedString,
    value_text: SharedString,
    identity: SharedString,
    focused: bool,
    selected: bool,
    frame: FrameRect,
}

fn active_popup_keyboard_target(
    nodes: &ModelRc<TemplatePaneNodeData>,
    interaction: &HostPaneInteractionStateData,
    bounds: &FrameRect,
) -> Option<PopupKeyboardTarget> {
    let mut fallback = None;
    for row in (0..nodes.row_count()).rev() {
        let Some(node) = nodes.row_data(row) else {
            continue;
        };
        let Some(target) = popup_keyboard_target_for_node(&node, interaction, bounds) else {
            continue;
        };
        let is_hovered_popup =
            node.control_id.as_str() == interaction.hovered_template_control_id.as_str();
        if is_hovered_popup || node.focused || node.selected || fallback.is_none() {
            fallback = Some(target);
        }
        if is_hovered_popup {
            break;
        }
    }
    fallback
}

fn popup_keyboard_target_for_node(
    node: &TemplatePaneNodeData,
    interaction: &HostPaneInteractionStateData,
    bounds: &FrameRect,
) -> Option<PopupKeyboardTarget> {
    if !node.popup_open || node.disabled || node.control_id.is_empty() {
        return None;
    }

    option_popup_keyboard_target(node, interaction, bounds)
        .or_else(|| menu_popup_keyboard_target(node, interaction))
}

fn option_popup_keyboard_target(
    node: &TemplatePaneNodeData,
    interaction: &HostPaneInteractionStateData,
    bounds: &FrameRect,
) -> Option<PopupKeyboardTarget> {
    let row_count = node.structured_options.row_count();
    if row_count == 0 {
        return None;
    }

    let action_id = if node.edit_action_id.is_empty() {
        node.action_id.clone()
    } else {
        node.edit_action_id.clone()
    };
    if action_id.is_empty() {
        return None;
    }

    let control_frame = frame_from_template_node(node);
    let rows: Vec<_> = (0..row_count)
        .filter_map(|row| {
            let option = node.structured_options.row_data(row)?;
            if option.disabled {
                return None;
            }
            Some(PopupKeyboardRow {
                action_id: action_id.clone(),
                value_text: option.id.clone(),
                identity: option.id,
                focused: option.focused || option.hovered || option.pressed,
                selected: option.selected || option.special,
                frame: dropdown_option_row_frame_within(&control_frame, row_count, row, bounds)?,
            })
        })
        .collect();
    let popup_frame = dropdown_option_popup_frame_within(&control_frame, row_count, bounds)
        .unwrap_or_else(|| control_frame);
    popup_keyboard_target_from_rows(node, "workbench_option", rows, popup_frame, interaction)
}

fn menu_popup_keyboard_target(
    node: &TemplatePaneNodeData,
    interaction: &HostPaneInteractionStateData,
) -> Option<PopupKeyboardTarget> {
    let row_count = node.structured_menu_items.row_count();
    if row_count == 0 {
        return None;
    }

    let menu_frame = frame_from_template_node(node);
    let rows: Vec<_> = (0..row_count)
        .filter_map(|row| {
            let item = node.structured_menu_items.row_data(row)?;
            if item.disabled || item.separator || item.action_id.is_empty() {
                return None;
            }
            Some(PopupKeyboardRow {
                action_id: item.action_id.clone(),
                value_text: item.label.clone(),
                identity: item.action_id,
                focused: item.focused || item.hovered || item.pressed,
                selected: item.checked,
                frame: menu_item_row_frame(&menu_frame, row_count, row)?,
            })
        })
        .collect();
    popup_keyboard_target_from_rows(node, "workbench_menu_item", rows, menu_frame, interaction)
}

fn popup_keyboard_target_from_rows(
    node: &TemplatePaneNodeData,
    dispatch_kind: &str,
    rows: Vec<PopupKeyboardRow>,
    popup_frame: FrameRect,
    interaction: &HostPaneInteractionStateData,
) -> Option<PopupKeyboardTarget> {
    if rows.is_empty() {
        return None;
    }

    let current_index = active_row_index(&rows, dispatch_kind, interaction);
    let current_row = rows.get(current_index).cloned();
    let current_frame = current_row
        .as_ref()
        .map(|row| row.frame.clone())
        .unwrap_or_else(|| frame_from_template_node(node));
    Some(PopupKeyboardTarget {
        control_id: node.control_id.clone(),
        dispatch_kind: dispatch_kind.into(),
        rows,
        current_index,
        current_row,
        current_frame,
        popup_frame,
    })
}

fn active_row_index(
    rows: &[PopupKeyboardRow],
    dispatch_kind: &str,
    interaction: &HostPaneInteractionStateData,
) -> usize {
    let interaction_identity = match dispatch_kind {
        "workbench_option" => interaction.hovered_template_value_text.as_str(),
        "workbench_menu_item" => interaction.hovered_template_action_id.as_str(),
        _ => "",
    };
    if !interaction_identity.is_empty() {
        if let Some(index) = rows
            .iter()
            .position(|row| row.identity.as_str() == interaction_identity)
        {
            return index;
        }
    }

    rows.iter()
        .position(|row| row.focused)
        .or_else(|| rows.iter().position(|row| row.selected))
        .unwrap_or(0)
}
