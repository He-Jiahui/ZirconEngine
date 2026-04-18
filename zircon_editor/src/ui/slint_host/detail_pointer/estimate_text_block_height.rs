use super::estimate_wrapped_line_count::estimate_wrapped_line_count;

pub(super) fn estimate_text_block_height(
    text: &str,
    width: f32,
    char_width: f32,
    line_height: f32,
) -> f32 {
    estimate_wrapped_line_count(text, width, char_width) as f32 * line_height
}
