use super::model::{PopupKeyboardRow, PopupKeyboardTarget};
use super::selection::popup_keyboard_target_from_rows;
use crate::ui::retained_host::host_contract::data::{
    HostPaneInteractionStateData, TemplatePaneNodeData,
};
use crate::ui::retained_host::host_contract::template_geometry::frame_from_template_node;
use crate::ui::retained_host::host_contract::template_popup_layout::menu_item_row_frame;

pub(in crate::ui::retained_host::host_contract) fn menu_popup_keyboard_target(
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
                source_index: None,
                frame: menu_item_row_frame(node, &menu_frame, row_count, row)?,
            })
        })
        .collect();
    popup_keyboard_target_from_rows(node, "workbench_menu_item", rows, menu_frame, interaction)
}
