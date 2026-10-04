const MIN_TEXT_METRIC_PX: f32 = 1.0;

pub(super) fn resolved_font_size(font_size: f32) -> f32 {
    font_size.max(MIN_TEXT_METRIC_PX)
}

pub(super) fn resolved_line_height(font_size: f32, line_height: f32) -> f32 {
    line_height.max(font_size).max(MIN_TEXT_METRIC_PX)
}

#[cfg(test)]
#[path = "tests/metrics.rs"]
mod tests;
