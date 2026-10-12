use zircon_runtime_interface::ui::surface::UiResolvedStyle;

const EMPTY_TEXT_MEASURE_WIDTH_PX: f32 = 0.0;

pub(super) fn should_measure_runtime_text(text: &str, font_size: f32) -> bool {
    !text.is_empty() && valid_runtime_font_size(font_size)
}

pub(super) fn empty_runtime_text_width() -> f32 {
    EMPTY_TEXT_MEASURE_WIDTH_PX
}

pub(super) fn measured_text_width(width: f32) -> f32 {
    width.max(EMPTY_TEXT_MEASURE_WIDTH_PX)
}

pub(super) fn resolved_runtime_font_size(font_size: f32) -> f32 {
    if valid_runtime_font_size(font_size) {
        font_size
    } else {
        UiResolvedStyle::DEFAULT_FONT_SIZE
    }
}

pub(super) fn resolved_runtime_line_height(font_size: f32, line_height: f32) -> f32 {
    if line_height.is_finite() && line_height > EMPTY_TEXT_MEASURE_WIDTH_PX {
        line_height
    } else {
        default_runtime_line_height(font_size)
    }
}

pub(super) fn default_runtime_line_height(font_size: f32) -> f32 {
    UiResolvedStyle::default_line_height(font_size)
}

fn valid_runtime_font_size(font_size: f32) -> bool {
    font_size.is_finite() && font_size > EMPTY_TEXT_MEASURE_WIDTH_PX
}

#[cfg(test)]
#[path = "tests/metrics.rs"]
mod tests;
