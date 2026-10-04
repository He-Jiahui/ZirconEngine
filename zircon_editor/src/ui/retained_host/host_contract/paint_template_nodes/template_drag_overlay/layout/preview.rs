//! 预览位置优先采用投影游标和偏移，没有游标时才使用节点矩形；内容区域受预览尺寸约束。
//! 调用方必须提供同一宿主物理坐标系中的游标与 fallback，尺寸退化时保持不绘制。

use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::metrics::DragOverlayMetrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn preview_frame(
    node: &TemplatePaneNodeData,
    fallback: &FrameRect,
) -> FrameRect {
    let width = node.drag_preview_width.max(0.0);
    let height = node.drag_preview_height.max(0.0);
    let width = if width > 0.0 { width } else { fallback.width };
    let height = if height > 0.0 {
        height
    } else {
        fallback.height
    };
    // TODO: [CR-EDITOR-PAINT-OVERLAY-0004] mount 投影缩放游标和目标坐标，但只给 node.frame/clip/popup anchor 加挂载原点；
    // 确认这组拖拽字段是否已是宿主绝对坐标。若属性来自挂载局部坐标，非零原点下预览和落点都会偏移。
    if node.has_drag_cursor {
        return FrameRect {
            x: node.drag_cursor_x + node.drag_offset_x,
            y: node.drag_cursor_y + node.drag_offset_y,
            width: width.max(0.0),
            height: height.max(0.0),
        };
    }
    FrameRect {
        x: fallback.x,
        y: fallback.y,
        width: width.max(0.0),
        height: height.max(0.0),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn preview_icon_frame(
    preview_rect: &FrameRect,
    metrics: &DragOverlayMetrics,
) -> FrameRect {
    let left = metrics.icon_left.min(preview_rect.width.max(0.0));
    let size = metrics
        .icon_size
        .min((preview_rect.width - left).max(0.0))
        .min(preview_rect.height.max(0.0));
    FrameRect {
        x: preview_rect.x + left,
        y: preview_rect.y + (preview_rect.height - size).max(0.0) * 0.5,
        width: size,
        height: size,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn preview_text_frame(
    preview_rect: &FrameRect,
    metrics: &DragOverlayMetrics,
) -> FrameRect {
    let text_left = preview_rect.x + metrics.text_left_with_icon.min(preview_rect.width.max(0.0));
    let right_inset = metrics
        .text_right_inset
        .min((preview_rect.x + preview_rect.width - text_left).max(0.0));
    let line_height = metrics.line_height.min(preview_rect.height.max(0.0));
    FrameRect {
        x: text_left,
        y: preview_rect.y + (preview_rect.height - line_height).max(0.0) * 0.5,
        width: (preview_rect.x + preview_rect.width - right_inset - text_left).max(0.0),
        height: line_height,
    }
}

#[cfg(test)]
#[path = "tests/preview.rs"]
mod tests;
