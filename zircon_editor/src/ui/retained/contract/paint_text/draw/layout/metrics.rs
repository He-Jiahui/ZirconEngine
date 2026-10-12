const LINE_VERTICAL_CENTER_FACTOR: f32 = 0.5;

pub(in crate::ui::retained_host::host_contract::paint_text::draw) fn centered_line_y(
    rect_y: f32,
    rect_height: f32,
    line_height: f32,
) -> f32 {
    rect_y + (rect_height - line_height).max(0.0) * LINE_VERTICAL_CENTER_FACTOR
}

#[cfg(test)]
#[path = "tests/metrics.rs"]
mod tests;
