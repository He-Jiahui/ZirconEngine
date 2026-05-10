use super::console_constants::{
    CONSOLE_BLOCK_GAP, CONSOLE_BODY_CHAR_WIDTH, CONSOLE_BODY_LINE_HEIGHT, CONSOLE_EMPTY_STATUS,
    CONSOLE_MONO_CHAR_WIDTH, CONSOLE_MONO_LINE_HEIGHT, CONSOLE_PADDING_BOTTOM, CONSOLE_PADDING_TOP,
    CONSOLE_TEXT_WIDTH_INSET,
};
use super::estimate_text_block_height::estimate_text_block_height;

pub(crate) fn console_content_extent(
    status_text: &str,
    pane_width: f32,
    show_empty: bool,
    empty_body: &str,
) -> f32 {
    let content_width = (pane_width - CONSOLE_TEXT_WIDTH_INSET).max(CONSOLE_BODY_CHAR_WIDTH);
    let status_block = if show_empty {
        if status_text.trim().is_empty() {
            CONSOLE_EMPTY_STATUS.to_string()
        } else {
            format!("> {status_text}")
        }
    } else {
        status_text.to_string()
    };
    let mut height = CONSOLE_PADDING_TOP
        + estimate_text_block_height(
            &status_block,
            content_width,
            CONSOLE_MONO_CHAR_WIDTH,
            CONSOLE_MONO_LINE_HEIGHT,
        )
        + CONSOLE_PADDING_BOTTOM;
    if show_empty && !empty_body.trim().is_empty() {
        height += CONSOLE_BLOCK_GAP
            + estimate_text_block_height(
                empty_body,
                content_width,
                CONSOLE_BODY_CHAR_WIDTH,
                CONSOLE_BODY_LINE_HEIGHT,
            );
    }
    height
}
