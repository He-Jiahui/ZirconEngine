use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::metrics::{
    SLIDER_HORIZONTAL_INSET, SLIDER_LABEL_GAP, SLIDER_LABEL_WIDTH, SLIDER_TRACK_HEIGHT,
    SLIDER_VALUE_GAP,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_track_rect(
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

fn slider_track_offset_x(node: &TemplatePaneNodeData) -> f32 {
    node.layout_content_offset_x
}

fn slider_track_width_delta(node: &TemplatePaneNodeData) -> f32 {
    node.layout_first_cell_offset_x
}
