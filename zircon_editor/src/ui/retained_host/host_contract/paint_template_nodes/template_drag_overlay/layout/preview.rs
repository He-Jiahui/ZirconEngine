use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::metrics::{ICON_LEFT, ICON_SIZE, LINE_HEIGHT, TEXT_LEFT_WITH_ICON, TEXT_RIGHT_INSET};

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
            width: width.max(1.0),
            height: height.max(1.0),
        };
    }
    FrameRect {
        x: fallback.x,
        y: fallback.y,
        width: width.max(1.0),
        height: height.max(1.0),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn preview_icon_frame(
    preview_rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: preview_rect.x + ICON_LEFT,
        y: preview_rect.y + (preview_rect.height - ICON_SIZE).max(0.0) * 0.5,
        width: ICON_SIZE,
        height: ICON_SIZE,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn preview_text_frame(
    preview_rect: &FrameRect,
) -> FrameRect {
    let text_left = preview_rect.x + TEXT_LEFT_WITH_ICON;
    FrameRect {
        x: text_left,
        y: preview_rect.y + (preview_rect.height - LINE_HEIGHT).max(0.0) * 0.5,
        width: (preview_rect.x + preview_rect.width - TEXT_RIGHT_INSET - text_left).max(1.0),
        height: LINE_HEIGHT,
    }
}
