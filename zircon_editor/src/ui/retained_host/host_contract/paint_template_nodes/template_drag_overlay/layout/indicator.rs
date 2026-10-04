//! 消费目标矩形和边缘语义，生成落点反馈区域；未知边缘或退化目标表示没有可绘制指示器。
//! 目标坐标必须与预览及 clip 属于同一宿主物理坐标系。

use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::metrics::DragOverlayMetrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn indicator_frame(
    node: &TemplatePaneNodeData,
    metrics: &DragOverlayMetrics,
) -> Option<FrameRect> {
    if !node.has_drop_target {
        return None;
    }
    let width = node.drop_target_width.max(0.0);
    let height = node.drop_target_height.max(0.0);
    let thickness = metrics.indicator_thickness.min(width.min(height)).max(0.0);
    if width <= 0.0 || height <= 0.0 || thickness <= 0.0 {
        return None;
    }
    match node.drop_indicator_edge.as_str() {
        "top" => Some(FrameRect {
            x: node.drop_target_x,
            y: node.drop_target_y,
            width,
            height: thickness,
        }),
        "bottom" => Some(FrameRect {
            x: node.drop_target_x,
            y: node.drop_target_y + (height - thickness).max(0.0),
            width,
            height: thickness,
        }),
        "left" => Some(FrameRect {
            x: node.drop_target_x,
            y: node.drop_target_y,
            width: thickness,
            height,
        }),
        "right" => Some(FrameRect {
            x: node.drop_target_x + (width - thickness).max(0.0),
            y: node.drop_target_y,
            width: thickness,
            height,
        }),
        "inside" => Some(FrameRect {
            x: node.drop_target_x,
            y: node.drop_target_y,
            width,
            height,
        }),
        _ => None,
    }
}

#[cfg(test)]
#[path = "tests/indicator.rs"]
mod tests;
