use crate::ui::layouts::views::ViewTemplateNodeData;
use crate::ui::workbench::asset_content_layout::{
    BROWSER_CONTENT_LIST_ROW_HEIGHT, BROWSER_CONTENT_TABLE_CONTROL_ID,
    BROWSER_CONTENT_TABLE_HEADER_CONTROL_ID, BROWSER_CONTENT_TABLE_HEADER_HEIGHT,
};

use super::table_nodes::asset_table_row_index;

pub(super) fn collapse_compact_table_nodes(nodes: &mut [ViewTemplateNodeData], x: f32, y: f32) {
    for node in nodes.iter_mut() {
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
    let x = finite_coordinate(x);
    let y = finite_coordinate(y);
    let width = finite_non_negative(width);
    let height = finite_non_negative(height);
    for node in nodes.iter_mut() {
        if node.control_id == BROWSER_CONTENT_TABLE_HEADER_CONTROL_ID {
            node.frame.x = x;
            node.frame.y = y;
            node.frame.width = width;
            node.frame.height = BROWSER_CONTENT_TABLE_HEADER_HEIGHT;
        } else if node.control_id == BROWSER_CONTENT_TABLE_CONTROL_ID {
            node.value_number = BROWSER_CONTENT_LIST_ROW_HEIGHT * row_count as f32;
        } else if let Some(index) = asset_table_row_index(node.control_id.as_str()) {
            let row_offset = BROWSER_CONTENT_LIST_ROW_HEIGHT * index as f32;
            let available_row_height =
                finite_non_negative(height - BROWSER_CONTENT_TABLE_HEADER_HEIGHT - row_offset);
            node.frame.x = x;
            node.frame.y = y + BROWSER_CONTENT_TABLE_HEADER_HEIGHT + row_offset;
            node.frame.width = width;
            node.frame.height = (index < row_count
                && available_row_height >= BROWSER_CONTENT_LIST_ROW_HEIGHT)
                .then_some(BROWSER_CONTENT_LIST_ROW_HEIGHT)
                .unwrap_or(0.0);
        }
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

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn finite_coordinate(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
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
