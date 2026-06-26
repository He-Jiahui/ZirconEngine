use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::metrics::{tab_line_height, tab_text_inset_x, tab_underline_height};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tab_paint_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x + node.layout_offset_x,
        y: rect.y + node.layout_offset_y,
        width: rect.width,
        height: rect.height,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tab_underline_rect(
    rect: &FrameRect,
) -> FrameRect {
    let underline_height = tab_underline_height();
    FrameRect {
        x: rect.x,
        y: rect.y + (rect.height - underline_height).max(0.0),
        width: rect.width,
        height: underline_height,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tab_label_rect(
    rect: &FrameRect,
) -> FrameRect {
    let inset_x = tab_text_inset_x();
    let line_height = tab_line_height();
    FrameRect {
        x: rect.x + inset_x,
        y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
        width: (rect.width - inset_x * 2.0).max(1.0),
        height: line_height,
    }
}
