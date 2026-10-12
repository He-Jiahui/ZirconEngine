use super::super::data::HostWindowPresentationData;
use super::super::paint_frame::HostRgbaFrame;
use super::super::paint_primitives::{draw_border, draw_rect, draw_text_bars_clipped};
use super::button::draw_prompt_button;
use super::colors::close_prompt_palette;
use super::layout::prompt_text_layout;
use crate::ui::retained_host::host_contract::paint_theme::current_host_palette;

pub(in crate::ui::retained_host::host_contract) fn draw_close_prompt(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    let prompt = &presentation.close_prompt;
    if !prompt.visible {
        return;
    }
    let palette = close_prompt_palette(current_host_palette());
    let text_layout = prompt_text_layout(prompt);

    draw_rect(frame, prompt.overlay_frame.clone(), palette.overlay);
    draw_rect(frame, prompt.dialog_frame.clone(), palette.dialog);
    draw_border(frame, prompt.dialog_frame.clone(), palette.accent);

    draw_text_bars_clipped(
        frame,
        text_layout.title_x,
        text_layout.title_y,
        &prompt.title,
        Some(&prompt.dialog_frame),
        palette.text,
    );
    draw_text_bars_clipped(
        frame,
        text_layout.message_x,
        text_layout.message_y,
        &prompt.message,
        Some(&prompt.dialog_frame),
        palette.text_muted,
    );
    let details_frame = text_layout.details_frame;
    draw_rect(frame, details_frame.clone(), palette.dialog_inset);
    draw_text_bars_clipped(
        frame,
        text_layout.details_x,
        text_layout.details_y,
        &prompt.details,
        Some(&details_frame),
        palette.warning,
    );

    draw_prompt_button(
        frame,
        &prompt.save_button_frame,
        "Save",
        prompt.can_save,
        palette,
    );
    draw_prompt_button(
        frame,
        &prompt.discard_button_frame,
        "Discard",
        true,
        palette,
    );
    draw_prompt_button(frame, &prompt.cancel_button_frame, "Cancel", true, palette);
}
