use super::model::{PopupKeyboardRow, PopupKeyboardTarget};
use super::selection::popup_keyboard_target_from_rows;
use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostPaneInteractionStateData, TemplatePaneNodeData,
};
use crate::ui::retained_host::host_contract::template_geometry::frame_from_template_node;
use crate::ui::retained_host::host_contract::template_popup_layout::{
    dropdown_option_row_frame_within, template_option_popup_frame_within,
    template_option_row_frame_within, template_option_rows_use_projected_frame,
};

pub(in crate::ui::retained_host::host_contract) fn option_popup_keyboard_target(
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
                source_index: None,
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
