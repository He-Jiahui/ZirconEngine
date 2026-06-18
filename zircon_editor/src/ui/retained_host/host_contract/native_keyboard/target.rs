use super::super::data::{FrameRect, HostPaneInteractionStateData, TemplatePaneNodeData};
use super::super::template_geometry::{frame_from_template_node, template_popup_bounds};
use super::super::template_popup_layout::{
    dropdown_option_row_frame_within, menu_item_row_frame, template_option_popup_frame_within,
    template_option_row_frame_within, template_option_rows_use_projected_frame,
};
use super::super::window::UiHostWindow;
use super::commands::WorkbenchPopupKeyboardCommand;
use crate::ui::retained_host::primitives::{ModelRc, SharedString};

pub(super) fn active_popup_keyboard_target_for_ui(
    ui: &UiHostWindow,
) -> Option<PopupKeyboardTarget> {
    let presentation = ui.get_host_presentation();
    let interaction = ui.get_pane_interaction_state();
    let bounds = template_popup_bounds(
        &presentation.host_shell.native_window_bounds,
        &presentation.workbench_window_nodes,
    );
    active_popup_keyboard_target(&presentation.workbench_window_nodes, &interaction, &bounds)
}

pub(super) struct PopupKeyboardTarget {
    pub(super) control_id: SharedString,
    pub(super) dispatch_kind: SharedString,
    rows: Vec<PopupKeyboardRow>,
    current_index: usize,
    pub(super) current_row: Option<PopupKeyboardRow>,
    pub(super) current_frame: FrameRect,
    pub(super) popup_frame: FrameRect,
}

impl PopupKeyboardTarget {
    pub(super) fn next_row(
        &self,
        command: WorkbenchPopupKeyboardCommand,
    ) -> Option<PopupKeyboardRow> {
        if self.rows.is_empty() {
            return None;
        }
        let next_index = match command {
            WorkbenchPopupKeyboardCommand::Next => (self.current_index + 1) % self.rows.len(),
            WorkbenchPopupKeyboardCommand::Previous => {
                (self.current_index + self.rows.len() - 1) % self.rows.len()
            }
            WorkbenchPopupKeyboardCommand::First => 0,
            WorkbenchPopupKeyboardCommand::Last => self.rows.len() - 1,
            WorkbenchPopupKeyboardCommand::Accept | WorkbenchPopupKeyboardCommand::Cancel => {
                self.current_index
            }
        };
        self.rows.get(next_index).cloned()
    }

    pub(super) fn text_search_row(&self, text: &str) -> Option<PopupKeyboardRow> {
        if self.rows.is_empty() {
            return None;
        }
        let query = normalized_popup_text_query(text)?;
        let start_index = (self.current_index + 1) % self.rows.len();
        self.rows
            .iter()
            .cycle()
            .skip(start_index)
            .take(self.rows.len())
            .find(|row| row.matches_text_query(&query))
            .cloned()
    }
}

#[derive(Clone)]
pub(super) struct PopupKeyboardRow {
    pub(super) action_id: SharedString,
    pub(super) value_text: SharedString,
    identity: SharedString,
    search_text: SharedString,
    focused: bool,
    selected: bool,
    pub(super) frame: FrameRect,
}

impl PopupKeyboardRow {
    fn matches_text_query(&self, query: &str) -> bool {
        popup_text_starts_with(&self.search_text, query)
            || popup_text_starts_with(&self.value_text, query)
            || popup_text_starts_with(&self.identity, query)
    }
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

fn option_keyboard_row_frame_within(
    node: &TemplatePaneNodeData,
    control_frame: &FrameRect,
    row_count: usize,
    row: usize,
    bounds: &FrameRect,
) -> Option<FrameRect> {
    if template_option_rows_use_projected_frame(node) {
        template_option_row_frame_within(node, control_frame, row_count, row, bounds)
    } else {
        dropdown_option_row_frame_within(control_frame, row_count, row, bounds)
    }
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
                identity: option.id.clone(),
                search_text: if option.label.is_empty() {
                    option.id
                } else {
                    option.label
                },
                focused: option.focused || option.hovered || option.pressed,
                selected: option.selected || option.special,
                frame: option_keyboard_row_frame_within(
                    node,
                    &control_frame,
                    row_count,
                    row,
                    bounds,
                )?,
            })
        })
        .collect();
    let popup_frame = template_option_popup_frame_within(node, &control_frame, row_count, bounds)
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
                identity: item.action_id.clone(),
                search_text: item.label,
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

fn normalized_popup_text_query(text: &str) -> Option<String> {
    let query = text.trim().to_lowercase();
    if query.is_empty() {
        None
    } else {
        Some(query)
    }
}

fn popup_text_starts_with(value: &SharedString, query: &str) -> bool {
    value.as_str().to_lowercase().starts_with(query)
}
