use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::metrics::{
    SELECTION_LABEL_GAP, SELECTION_MARK_INSET_X, SELECTION_MARK_SIZE, SELECTION_TEXT_INSET_Y,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn leading_mark_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    let mark_size = selection_mark_size(node);
    FrameRect {
        x: rect.x + SELECTION_MARK_INSET_X,
        y: rect.y + (rect.height - mark_size).max(0.0) * 0.5,
        width: mark_size,
        height: mark_size,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn label_rect_after_mark(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    mark: &FrameRect,
) -> FrameRect {
    let x = mark.x + mark.width + selection_label_gap(node);
    FrameRect {
        x,
        y: rect.y + SELECTION_TEXT_INSET_Y,
        width: (rect.x + rect.width - x - SELECTION_MARK_INSET_X).max(1.0),
        height: (rect.height - SELECTION_TEXT_INSET_Y * 2.0).max(1.0),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn selection_label_gap(
    node: &TemplatePaneNodeData,
) -> f32 {
    if node.layout_content_offset_x > 0.0 {
        node.layout_content_offset_x
    } else {
        SELECTION_LABEL_GAP
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn centered_square(
    rect: &FrameRect,
    size: f32,
) -> FrameRect {
    let size = size.min(rect.width).min(rect.height).max(1.0);
    FrameRect {
        x: rect.x + (rect.width - size).max(0.0) * 0.5,
        y: rect.y + (rect.height - size).max(0.0) * 0.5,
        width: size,
        height: size,
    }
}

fn selection_mark_size(node: &TemplatePaneNodeData) -> f32 {
    if node.layout_icon_size > 0.0 {
        node.layout_icon_size
    } else {
        SELECTION_MARK_SIZE
    }
}
