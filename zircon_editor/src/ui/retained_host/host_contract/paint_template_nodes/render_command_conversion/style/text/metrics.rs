const MIN_MEASURED_TEXT_WIDTH_PX: f32 = 0.0;
const TEXT_CENTER_ALIGNMENT_FACTOR: f32 = 0.5;

pub(super) fn measured_text_width(width: f32) -> f32 {
    width.max(MIN_MEASURED_TEXT_WIDTH_PX)
}

pub(super) fn center_aligned_text_x(frame_x: f32, frame_width: f32, measured_width: f32) -> f32 {
    frame_x + remaining_text_space(frame_width, measured_width) * TEXT_CENTER_ALIGNMENT_FACTOR
}

pub(super) fn right_aligned_text_x(frame_x: f32, frame_width: f32, measured_width: f32) -> f32 {
    frame_x + remaining_text_space(frame_width, measured_width)
}

fn remaining_text_space(frame_width: f32, measured_width: f32) -> f32 {
    (frame_width - measured_width).max(MIN_MEASURED_TEXT_WIDTH_PX)
}

#[cfg(test)]
#[path = "tests/metrics.rs"]
mod tests;
