use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::metrics::{
    TREE_BASE_INSET_X, TREE_DISCLOSURE_SIZE, TREE_GUIDE_OFFSET_X, TREE_GUIDE_STEP, TREE_ICON_SIZE,
    TREE_TEXT_GAP,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_disclosure_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    let indent = if node.tree_indent_px.is_finite() && node.tree_indent_px > 0.0 {
        node.tree_indent_px
    } else {
        node.tree_depth.max(0) as f32 * TREE_GUIDE_STEP
    };
    FrameRect {
        x: rect.x + TREE_BASE_INSET_X + indent,
        y: rect.y + (rect.height - TREE_DISCLOSURE_SIZE).max(0.0) * 0.5,
        width: TREE_DISCLOSURE_SIZE,
        height: TREE_DISCLOSURE_SIZE,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_icon_rect(
    disclosure: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: disclosure.x + disclosure.width + TREE_TEXT_GAP,
        y: disclosure.y + (disclosure.height - TREE_ICON_SIZE).max(0.0) * 0.5,
        width: TREE_ICON_SIZE,
        height: TREE_ICON_SIZE,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_guide_x(
    rect: &FrameRect,
    level: usize,
) -> f32 {
    rect.x + TREE_BASE_INSET_X + TREE_GUIDE_OFFSET_X + level as f32 * TREE_GUIDE_STEP
}
