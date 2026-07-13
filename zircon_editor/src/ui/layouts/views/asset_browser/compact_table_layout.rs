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
        set_frame(
            nodes,
            &asset_table_row_control_id(index),
            x,
            row_y,
            width,
            BROWSER_CONTENT_LIST_ROW_HEIGHT,
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
    content_height.min(available_height.max(BROWSER_CONTENT_TABLE_HEADER_HEIGHT))
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
        node.frame.x = x;
        node.frame.y = y;
        node.frame.width = width.max(0.0);
        node.frame.height = height.max(0.0);
    }
}
