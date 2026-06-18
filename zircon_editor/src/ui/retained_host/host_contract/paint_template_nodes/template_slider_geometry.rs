use super::super::data::{FrameRect, TemplatePaneNodeData};

pub(super) const SLIDER_TRACK_HEIGHT: f32 = 3.0;
pub(super) const SLIDER_TRACK_RADIUS: f32 = 2.0;
pub(super) const SLIDER_THUMB_SIZE: f32 = 11.0;
pub(super) const SLIDER_THUMB_HALO_SIZE: f32 = 20.0;
pub(super) const SLIDER_HORIZONTAL_INSET: f32 = 8.0;
pub(super) const SLIDER_LABEL_WIDTH: f32 = 50.0;
pub(super) const SLIDER_LABEL_GAP: f32 = 12.0;
pub(super) const SLIDER_VALUE_WIDTH: f32 = 44.0;
pub(super) const SLIDER_VALUE_GAP: f32 = 10.0;
pub(super) const SLIDER_FONT_SIZE: f32 = 11.0;
pub(super) const SLIDER_LINE_HEIGHT: f32 = SLIDER_FONT_SIZE * 1.2;

pub(super) fn slider_track_rect(
    rect: &FrameRect,
    value_rect: Option<&FrameRect>,
    has_label: bool,
    node: &TemplatePaneNodeData,
) -> FrameRect {
    let label_lane_width = if has_label {
        SLIDER_LABEL_WIDTH + SLIDER_LABEL_GAP
    } else {
        0.0
    };
    let left = rect.x + label_lane_width + SLIDER_HORIZONTAL_INSET + slider_track_offset_x(node);
    let right = (value_rect
        .map(|value| value.x - SLIDER_VALUE_GAP)
        .unwrap_or(rect.x + rect.width - SLIDER_HORIZONTAL_INSET)
        + slider_track_width_delta(node))
    .max(left);
    FrameRect {
        x: left,
        y: rect.y + (rect.height - SLIDER_TRACK_HEIGHT).max(0.0) * 0.5,
        width: right - left,
        height: SLIDER_TRACK_HEIGHT,
    }
}

pub(super) fn slider_value_rect(rect: &FrameRect) -> Option<FrameRect> {
    if rect.width < 132.0 {
        return None;
    }
    let height = (rect.height - 6.0).clamp(18.0, 24.0);
    Some(FrameRect {
        x: rect.x + rect.width - SLIDER_HORIZONTAL_INSET - SLIDER_VALUE_WIDTH,
        y: rect.y + (rect.height - height).max(0.0) * 0.5,
        width: SLIDER_VALUE_WIDTH,
        height,
    })
}

pub(super) fn slider_range_min_value_rect(
    rect: &FrameRect,
    track_rect: &FrameRect,
) -> Option<FrameRect> {
    if rect.height < 42.0 || track_rect.width < SLIDER_VALUE_WIDTH {
        return None;
    }
    Some(FrameRect {
        x: track_rect.x,
        y: track_rect.y + 10.0,
        width: SLIDER_VALUE_WIDTH,
        height: 20.0,
    })
}

pub(super) fn slider_percent(node: &TemplatePaneNodeData) -> f32 {
    if node.value_percent.is_finite() {
        node.value_percent.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

pub(super) fn slider_label(node: &TemplatePaneNodeData) -> Option<String> {
    let label = node.label_text.trim();
    (!label.is_empty()).then(|| label.to_owned())
}

pub(super) fn slider_value_label(node: &TemplatePaneNodeData, percent: f32) -> String {
    let value = node.value_text.trim();
    if value.is_empty() {
        format!("{:.2}", percent.clamp(0.0, 1.0))
    } else {
        value.to_owned()
    }
}

pub(super) fn slider_range_min_label(percent: f32) -> String {
    format!("{:.2}", percent.clamp(0.0, 1.0))
}

pub(super) fn slider_range_min_percent(node: &TemplatePaneNodeData) -> Option<f32> {
    let is_range_row = node.control_id.as_str().contains("RangeSlider");
    if !is_range_row && node.layout_second_cell_offset_x <= 0.0 {
        return None;
    }
    Some(slider_declared_percent(node.layout_second_cell_offset_x))
}

pub(super) fn slider_tick_count(node: &TemplatePaneNodeData) -> Option<usize> {
    let declared = node.layout_third_cell_offset_x.round() as usize;
    if declared >= 2 {
        Some(declared)
    } else if node.control_id.as_str().contains("StepsSlider") {
        Some(5)
    } else {
        None
    }
}

pub(super) fn slider_fill_span(percent: f32, range_min_percent: Option<f32>) -> (f32, f32) {
    let end = percent.clamp(0.0, 1.0);
    let start = range_min_percent.unwrap_or(0.0).clamp(0.0, 1.0);
    if start <= end {
        (start, end)
    } else {
        (end, start)
    }
}

pub(super) fn slider_thumb_size(node: &TemplatePaneNodeData) -> f32 {
    if node.layout_icon_size > 0.0 {
        node.layout_icon_size
    } else {
        SLIDER_THUMB_SIZE
    }
}

pub(super) fn centered_rect(center_x: f32, center_y: f32, size: f32) -> FrameRect {
    FrameRect {
        x: center_x - size * 0.5,
        y: center_y - size * 0.5,
        width: size,
        height: size,
    }
}

pub(super) fn pixel_aligned_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: rect.width.round().max(1.0),
        height: rect.height.round().max(1.0),
    }
}

fn slider_declared_percent(value: f32) -> f32 {
    if value > 1.0 {
        (value / 100.0).clamp(0.0, 1.0)
    } else {
        value.clamp(0.0, 1.0)
    }
}

fn slider_track_offset_x(node: &TemplatePaneNodeData) -> f32 {
    node.layout_content_offset_x
}

fn slider_track_width_delta(node: &TemplatePaneNodeData) -> f32 {
    node.layout_first_cell_offset_x
}
