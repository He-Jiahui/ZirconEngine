use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::metrics::tree_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_disclosure_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    let metrics = tree_metrics();
    let indent = if node.tree_indent_px.is_finite() && node.tree_indent_px > 0.0 {
        node.tree_indent_px
    } else {
        node.tree_depth.max(0) as f32 * metrics.tree_guide_step
    };
    FrameRect {
        x: rect.x + metrics.tree_base_inset_x + indent,
        y: rect.y + (rect.height - metrics.tree_disclosure_size).max(0.0) * 0.5,
        width: metrics.tree_disclosure_size,
        height: metrics.tree_disclosure_size,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_icon_rect(
    disclosure: &FrameRect,
) -> FrameRect {
    let metrics = tree_metrics();
    FrameRect {
        x: disclosure.x + disclosure.width + metrics.tree_text_gap,
        y: disclosure.y + (disclosure.height - metrics.tree_icon_size).max(0.0) * 0.5,
        width: metrics.tree_icon_size,
        height: metrics.tree_icon_size,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_guide_x(
    rect: &FrameRect,
    level: usize,
) -> f32 {
    let metrics = tree_metrics();
    rect.x
        + metrics.tree_base_inset_x
        + metrics.tree_guide_offset_x
        + level as f32 * metrics.tree_guide_step
}
