use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::metrics::{TAB_TEXT_INSET_X, TAB_UNDERLINE_HEIGHT};

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
    FrameRect {
        x: rect.x,
        y: rect.y + (rect.height - TAB_UNDERLINE_HEIGHT).max(0.0),
        width: rect.width,
        height: TAB_UNDERLINE_HEIGHT,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tab_label_rect(
    rect: &FrameRect,
) -> FrameRect {
    let line_height = super::metrics::tab_line_height();
    FrameRect {
        x: rect.x + TAB_TEXT_INSET_X,
        y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
        width: (rect.width - TAB_TEXT_INSET_X * 2.0).max(1.0),
        height: line_height,
    }
}
