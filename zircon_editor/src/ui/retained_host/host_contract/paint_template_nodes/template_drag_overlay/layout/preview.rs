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
mod tests {
    use super::*;

    fn metrics() -> DragOverlayMetrics {
        DragOverlayMetrics {
            border_width: 1.0,
            preview_radius: 4.0,
            icon_radius: 4.0,
            font_size: 13.33,
            line_height: 16.0,
            icon_left: 12.0,
            icon_size: 16.0,
            text_left_with_icon: 35.0,
            text_right_inset: 12.0,
            indicator_thickness: 2.0,
        }
    }

    #[test]
    fn preview_content_stays_inside_a_narrow_short_drag_frame() {
        let preview = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 18.0,
            height: 8.0,
        };
        let metrics = metrics();

        assert_contained(preview_icon_frame(&preview, &metrics), &preview);
        assert_contained(preview_text_frame(&preview, &metrics), &preview);
    }

    #[test]
    fn preview_frame_keeps_a_collapsed_fallback_collapsed() {
        let node = TemplatePaneNodeData::default();
        let fallback = FrameRect {
            x: 1.0,
            y: 2.0,
            width: 0.0,
            height: 0.0,
        };

        assert_eq!(preview_frame(&node, &fallback), fallback);
    }

    fn assert_contained(rect: FrameRect, parent: &FrameRect) {
        let epsilon = 0.000_1;
        assert!(rect.x >= parent.x - epsilon);
        assert!(rect.y >= parent.y - epsilon);
        assert!(rect.x + rect.width <= parent.x + parent.width + epsilon);
        assert!(rect.y + rect.height <= parent.y + parent.height + epsilon);
    }
}
