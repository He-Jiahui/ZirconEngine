use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::metrics::{SEGMENT_GROUP_LABEL_GAP, SEGMENT_GROUP_LABEL_HEIGHT};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segmented_body_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    let label_block_height = if node.label_text.trim().is_empty() {
        0.0
    } else {
        SEGMENT_GROUP_LABEL_HEIGHT + SEGMENT_GROUP_LABEL_GAP
    };

    FrameRect {
        x: rect.x + node.layout_offset_x,
        y: rect.y + label_block_height + node.layout_offset_y,
        width: rect.width,
        height: (rect.height - label_block_height).max(1.0),
    }
}
