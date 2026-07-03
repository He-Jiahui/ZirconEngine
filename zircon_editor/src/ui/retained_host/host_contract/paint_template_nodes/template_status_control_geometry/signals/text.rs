use super::super::metrics::status_line_height;
use super::constants::status_signal_metrics;
use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn status_signal_text_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    icon: &FrameRect,
) -> FrameRect {
    let line_height = status_line_height();
    let text_gap = status_signal_text_gap(node);
    FrameRect {
        x: icon.x + icon.width + text_gap,
        y: rect.y + node.layout_offset_y + (rect.height - line_height).max(0.0) * 0.5,
        width: (rect.x + rect.width - icon.x - icon.width - text_gap).max(1.0),
        height: line_height,
    }
}

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn status_signal_text_gap(
    node: &TemplatePaneNodeData,
) -> f32 {
    resolved_status_signal_text_gap(node)
}

#[cfg(not(test))]
fn status_signal_text_gap(node: &TemplatePaneNodeData) -> f32 {
    resolved_status_signal_text_gap(node)
}

fn resolved_status_signal_text_gap(node: &TemplatePaneNodeData) -> f32 {
    if node.layout_content_offset_x > 0.0 {
        node.layout_content_offset_x
    } else {
        status_signal_metrics().signal_text_gap
    }
}
