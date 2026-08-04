use crate::ui::layouts::views::ViewTemplateNodeData;
use crate::ui::workbench::asset_content_layout::{
    BROWSER_CONTENT_LIST_ROW_HEIGHT, BROWSER_CONTENT_TABLE_CONTROL_ID,
    BROWSER_CONTENT_TABLE_HEADER_CONTROL_ID, BROWSER_CONTENT_TABLE_HEADER_HEIGHT,
};

use super::table_nodes::{asset_table_row_control_id, asset_table_row_index};

pub(super) fn collapse_compact_table_nodes(nodes: &mut [ViewTemplateNodeData], x: f32, y: f32) {
    for node in nodes {
        if node.control_id == BROWSER_CONTENT_TABLE_CONTROL_ID
            || node.control_id == BROWSER_CONTENT_TABLE_HEADER_CONTROL_ID
            || asset_table_row_index(node.control_id.as_str()).is_some()
        {
            node.frame.x = x;
            node.frame.y = y;
            node.frame.width = 0.0;
            node.frame.height = 0.0;
        }
    }
}

pub(super) fn apply_compact_table_layout(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    row_count: usize,
) {
    set_frame(
        nodes,
        BROWSER_CONTENT_TABLE_HEADER_CONTROL_ID,
        x,
        y,
        width,
        BROWSER_CONTENT_TABLE_HEADER_HEIGHT,
    );
    for index in 0..row_count {
        let row_y = y
            + BROWSER_CONTENT_TABLE_HEADER_HEIGHT
            + BROWSER_CONTENT_LIST_ROW_HEIGHT * index as f32;
        let available_row_height = finite_non_negative(
            height
                - BROWSER_CONTENT_TABLE_HEADER_HEIGHT
                - BROWSER_CONTENT_LIST_ROW_HEIGHT * index as f32,
        );
        let row_height = (available_row_height >= BROWSER_CONTENT_LIST_ROW_HEIGHT)
            .then_some(BROWSER_CONTENT_LIST_ROW_HEIGHT)
            .unwrap_or(0.0);
        set_frame(
            nodes,
            &asset_table_row_control_id(index),
            x,
            row_y,
            width,
            row_height,
        );
    }
    if let Some(panel) = nodes
        .iter_mut()
        .find(|node| node.control_id == BROWSER_CONTENT_TABLE_CONTROL_ID)
    {
        panel.value_number = BROWSER_CONTENT_LIST_ROW_HEIGHT * row_count as f32;
    }
}

pub(super) fn compact_table_stack_height(available_height: f32, row_count: usize) -> f32 {
    let content_height =
        BROWSER_CONTENT_TABLE_HEADER_HEIGHT + BROWSER_CONTENT_LIST_ROW_HEIGHT * row_count as f32;
    let available_height = finite_non_negative(available_height);
    if available_height < BROWSER_CONTENT_TABLE_HEADER_HEIGHT {
        0.0
    } else {
        content_height.min(available_height)
    }
}

pub(super) fn asset_table_row_count(nodes: &[ViewTemplateNodeData]) -> usize {
    nodes
        .iter()
        .filter(|node| asset_table_row_index(node.control_id.as_str()).is_some())
        .count()
}

fn set_frame(
    nodes: &mut [ViewTemplateNodeData],
    control_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    if let Some(node) = nodes.iter_mut().find(|node| node.control_id == control_id) {
        node.frame.x = finite_coordinate(x);
        node.frame.y = finite_coordinate(y);
        node.frame.width = finite_non_negative(width);
        node.frame.height = finite_non_negative(height);
    }
}

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn finite_coordinate(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_stack_collapses_without_room_for_its_header() {
        assert_eq!(
            compact_table_stack_height(BROWSER_CONTENT_TABLE_HEADER_HEIGHT - 1.0, 4),
            0.0
        );
        assert_eq!(compact_table_stack_height(f32::NAN, 4), 0.0);
    }

    #[test]
    fn table_stack_never_exceeds_its_available_height() {
        let height = compact_table_stack_height(BROWSER_CONTENT_TABLE_HEADER_HEIGHT + 4.0, 4);
        assert_eq!(height, BROWSER_CONTENT_TABLE_HEADER_HEIGHT + 4.0);
    }
}
