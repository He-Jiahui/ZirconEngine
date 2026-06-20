use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::metrics::{
    SELECTION_MARK_INSET_X, TOGGLE_RIGHT_INSET, TOGGLE_THUMB_INSET, TOGGLE_THUMB_SIZE,
    TOGGLE_TRACK_HEIGHT, TOGGLE_TRACK_WIDTH,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn toggle_track_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    let track_width =
        toggle_track_width(node).min((rect.width - SELECTION_MARK_INSET_X * 2.0).max(1.0));
    let track_height = toggle_track_height(node).min(rect.height.max(1.0));
    FrameRect {
        x: (rect.x + rect.width - TOGGLE_RIGHT_INSET - track_width).max(rect.x),
        y: rect.y + (rect.height - track_height).max(0.0) * 0.5,
        width: track_width,
        height: track_height,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn toggle_thumb_rect(
    node: &TemplatePaneNodeData,
    track: &FrameRect,
) -> FrameRect {
    let thumb_size = toggle_thumb_size(node)
        .min(track.width)
        .min(track.height)
        .max(1.0);
    let available = (track.width - thumb_size - TOGGLE_THUMB_INSET * 2.0).max(0.0);
    let offset = if node.checked || node.selected {
        available
    } else {
        0.0
    };
    FrameRect {
        x: track.x + TOGGLE_THUMB_INSET + offset,
        y: track.y + (track.height - thumb_size).max(0.0) * 0.5,
        width: thumb_size,
        height: thumb_size,
    }
}

fn toggle_track_width(node: &TemplatePaneNodeData) -> f32 {
    if node.value_number > 0.0 {
        node.value_number
    } else {
        TOGGLE_TRACK_WIDTH
    }
}

fn toggle_track_height(node: &TemplatePaneNodeData) -> f32 {
    if node.layout_content_offset_y > 0.0 {
        node.layout_content_offset_y
    } else {
        TOGGLE_TRACK_HEIGHT
    }
}

fn toggle_thumb_size(node: &TemplatePaneNodeData) -> f32 {
    if node.layout_icon_size > 0.0 {
        node.layout_icon_size
    } else {
        TOGGLE_THUMB_SIZE
    }
}
