use super::model::{PopupKeyboardRow, PopupKeyboardTarget};
use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostPaneInteractionStateData, TemplatePaneNodeData,
};
use crate::ui::retained_host::host_contract::template_geometry::frame_from_template_node;

pub(in crate::ui::retained_host::host_contract) fn popup_keyboard_target_from_rows(
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
