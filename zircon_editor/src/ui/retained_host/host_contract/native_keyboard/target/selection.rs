use super::model::{PopupKeyboardRow, PopupKeyboardTarget};
use crate::ui::retained_host::asset_control_ids::asset_dispatch_source;
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
    let window_count = usize::try_from(node.pagination_page_size)
        .ok()
        .filter(|count| *count > 0)
        .unwrap_or(rows.len());
    let window_offset = usize::try_from(node.virtualization_visible_start).unwrap_or(0);
    // A cached visible window can arrive before its total-count projection catches up.
    let total_count = usize::try_from(node.virtualization_total_count)
        .unwrap_or(rows.len())
        .max(window_offset.saturating_add(rows.len()));
    Some(PopupKeyboardTarget {
        control_id: node.control_id.clone(),
        dispatch_kind: dispatch_kind.into(),
        rows,
        current_index,
        current_row,
        current_frame,
        popup_frame,
        window_offset,
        window_count,
        total_count,
        window_navigation_enabled: node.control_id.as_str() == "WorkbenchCommandPalette"
            && node.virtualization_enabled,
        window_query: node.search_query.clone(),
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
        kind if asset_dispatch_source(kind).is_some() => {
            interaction.hovered_template_value_text.as_str()
        }
        _ => "",
    };
    preferred_row_index(rows, interaction_identity)
}

fn preferred_row_index(rows: &[PopupKeyboardRow], interaction_identity: &str) -> usize {
    let mut focused_index = None;
    let mut selected_index = None;
    for (index, row) in rows.iter().enumerate() {
        if !interaction_identity.is_empty() && row.identity.as_str() == interaction_identity {
            return index;
        }
        if focused_index.is_none() && row.focused {
            focused_index = Some(index);
        }
        if selected_index.is_none() && row.selected {
            selected_index = Some(index);
        }
    }
    focused_index.or(selected_index).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtualized_target_total_covers_its_visible_window_when_projection_lags() {
        let mut node = TemplatePaneNodeData::default();
        node.control_id = "WorkbenchCommandPalette".into();
        node.virtualization_enabled = true;
        node.pagination_page_size = 12;
        node.virtualization_total_count = 1;
        node.virtualization_visible_start = 12;
        let rows = (0..12).map(row).collect();

        let target = popup_keyboard_target_from_rows(
            &node,
            "workbench_option",
            rows,
            FrameRect::default(),
            &HostPaneInteractionStateData::default(),
        )
        .expect("visible command rows should produce a keyboard target");

        assert_eq!(target.total_count, 24);
    }

    fn row(index: usize) -> PopupKeyboardRow {
        PopupKeyboardRow {
            action_id: format!("command_{index}").into(),
            value_text: format!("command_{index}").into(),
            identity: format!("command_{index}").into(),
            search_text: format!("Command {index}").into(),
            focused: false,
            selected: false,
            source_index: Some(index),
            frame: FrameRect::default(),
        }
    }
}

#[cfg(test)]
mod active_row_index_tests;
