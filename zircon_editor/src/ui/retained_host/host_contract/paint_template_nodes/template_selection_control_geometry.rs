use super::super::data::{FrameRect, TemplatePaneNodeData};

pub(super) const SELECTION_MARK_INSET_X: f32 = 10.0;
pub(super) const SELECTION_MARK_SIZE: f32 = 16.0;
pub(super) const SELECTION_LABEL_GAP: f32 = 9.0;
pub(super) const SELECTION_TEXT_INSET_Y: f32 = 5.0;
pub(super) const RADIO_DOT_SIZE: f32 = 7.0;
pub(super) const TOGGLE_TRACK_WIDTH: f32 = 34.0;
pub(super) const TOGGLE_TRACK_HEIGHT: f32 = 18.0;
pub(super) const TOGGLE_THUMB_SIZE: f32 = 14.0;
const TOGGLE_RIGHT_INSET: f32 = 8.0;
const TOGGLE_THUMB_INSET: f32 = 2.0;

pub(super) fn leading_mark_rect(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    let mark_size = selection_mark_size(node);
    FrameRect {
        x: rect.x + SELECTION_MARK_INSET_X,
        y: rect.y + (rect.height - mark_size).max(0.0) * 0.5,
        width: mark_size,
        height: mark_size,
    }
}

pub(super) fn label_rect_after_mark(
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

pub(super) fn selection_label_gap(node: &TemplatePaneNodeData) -> f32 {
    if node.layout_content_offset_x > 0.0 {
        node.layout_content_offset_x
    } else {
        SELECTION_LABEL_GAP
    }
}

pub(super) fn toggle_track_rect(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
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

pub(super) fn toggle_thumb_rect(node: &TemplatePaneNodeData, track: &FrameRect) -> FrameRect {
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

pub(super) fn radio_dot_size(node: &TemplatePaneNodeData) -> f32 {
    if node.value_number > 0.0 {
        node.value_number
    } else {
        RADIO_DOT_SIZE
    }
}

pub(super) fn centered_square(rect: &FrameRect, size: f32) -> FrameRect {
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
