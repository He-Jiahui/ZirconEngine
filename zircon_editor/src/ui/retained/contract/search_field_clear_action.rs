use zircon_runtime_interface::ui::layout::{UiFrame, UiPoint};

use super::current_host_metrics;

pub(crate) fn search_field_clear_action_frame(field: UiFrame) -> Option<UiFrame> {
    if !field.x.is_finite()
        || !field.y.is_finite()
        || !field.width.is_finite()
        || !field.height.is_finite()
        || field.width <= 0.0
        || field.height <= 0.0
    {
        return None;
    }

    let metrics = current_host_metrics();
    let size = (metrics.row_height - metrics.gap_l)
        .max(metrics.font_body)
        .min(field.height)
        .round();
    let right = field.x + field.width - metrics.input_pad[1];
    let frame = UiFrame::new(
        right - size,
        field.y + (field.height - size).max(0.0) * 0.5,
        size,
        size,
    );
    (frame.x >= field.x
        && frame.y >= field.y
        && frame.x + frame.width <= field.x + field.width
        && frame.y + frame.height <= field.y + field.height)
        .then_some(frame)
}

pub(crate) fn search_field_clear_action_hit_test(field: UiFrame, point: UiPoint) -> bool {
    let Some(action) = search_field_clear_action_frame(field) else {
        return false;
    };
    point.x >= action.x
        && point.y >= action.y
        && point.x <= action.x + action.width
        && point.y <= action.y + action.height
}

#[cfg(test)]
#[path = "tests/search_field_clear_action.rs"]
mod tests;
