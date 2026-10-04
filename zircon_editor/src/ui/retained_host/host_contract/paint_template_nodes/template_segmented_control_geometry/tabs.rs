//! 工作台页签的显示偏移、文字槽和底部强调线几何；与 segmented 主体共用宿主密度但消费不同角色。

use super::super::super::{
    data::{FrameRect, TemplatePaneNodeData},
    paint_geometry::bounded_extent,
};
use super::metrics::{tab_line_height, tab_text_inset_x, tab_underline_height};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tab_paint_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x + node.layout_offset_x,
        y: rect.y + node.layout_offset_y,
        width: bounded_extent(rect.width),
        height: bounded_extent(rect.height),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tab_underline_rect(
    rect: &FrameRect,
) -> FrameRect {
    let available_height = bounded_extent(rect.height);
    let underline_height = bounded_extent(tab_underline_height()).min(available_height);
    FrameRect {
        x: rect.x,
        y: rect.y + (available_height - underline_height).max(0.0),
        width: bounded_extent(rect.width),
        height: underline_height,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tab_label_rect(
    rect: &FrameRect,
) -> FrameRect {
    let inset_x = tab_text_inset_x();
    let line_height = tab_line_height();
    let available_height = bounded_extent(rect.height);
    let height = bounded_extent(line_height).min(available_height);
    FrameRect {
        x: rect.x + inset_x,
        y: rect.y + (available_height - height) * 0.5,
        width: bounded_extent(rect.width - inset_x * 2.0),
        height,
    }
}

#[cfg(test)]
#[path = "tests/tabs.rs"]
mod tests;
