//! 组标签存在时为其单独留出顶部高度，主体再应用节点显示偏移；调用方负责 clip 相交判断。

use super::super::super::super::{
    data::{FrameRect, TemplatePaneNodeData},
    paint_geometry::bounded_extent,
};
use super::super::metrics::{segment_group_label_gap, segment_group_label_height};

/// 调用方先绘组标题再绘主体；非空 label_text 才占用顶部高度，布局偏移只作用于主体。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segmented_body_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    let available_height = bounded_extent(rect.height);
    let label_block_height = if node.label_text.trim().is_empty() {
        0.0
    } else {
        segment_group_label_height() + segment_group_label_gap()
    }
    .min(available_height);

    FrameRect {
        x: rect.x + node.layout_offset_x,
        y: rect.y + label_block_height + node.layout_offset_y,
        width: bounded_extent(rect.width),
        height: available_height - label_block_height,
    }
}

#[cfg(test)]
#[path = "tests/body.rs"]
mod tests;
