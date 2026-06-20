use super::super::data::HostWindowPresentationData;
use super::super::paint_frame::HostRgbaFrame;
use super::super::paint_primitives::{draw_border, draw_rect, draw_text_bars_clipped};
use super::button::draw_prompt_button;
use super::colors::{ACCENT, DIALOG, DIALOG_INSET, MUTED, OVERLAY, TEXT, WARNING};
use super::layout::prompt_details_frame;

pub(in crate::ui::retained_host::host_contract) fn draw_close_prompt(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    let prompt = &presentation.close_prompt;
    if !prompt.visible {
        return;
    }

    draw_rect(frame, prompt.overlay_frame.clone(), OVERLAY);
    draw_rect(frame, prompt.dialog_frame.clone(), DIALOG);
    draw_border(frame, prompt.dialog_frame.clone(), ACCENT);

    draw_text_bars_clipped(
        frame,
        prompt.dialog_frame.x + 18.0,
        prompt.dialog_frame.y + 18.0,
        &prompt.title,
        Some(&prompt.dialog_frame),
        TEXT,
    );
    draw_text_bars_clipped(
        frame,
        prompt.dialog_frame.x + 18.0,
        prompt.dialog_frame.y + 48.0,
        &prompt.message,
        Some(&prompt.dialog_frame),
        MUTED,
    );
    let details_frame = prompt_details_frame(prompt);
    draw_rect(frame, details_frame.clone(), DIALOG_INSET);
    draw_text_bars_clipped(
        frame,
        prompt.dialog_frame.x + 24.0,
        prompt.dialog_frame.y + 86.0,
        &prompt.details,
        Some(&details_frame),
        WARNING,
    );

    draw_prompt_button(frame, &prompt.save_button_frame, "Save", prompt.can_save);
    draw_prompt_button(frame, &prompt.discard_button_frame, "Discard", true);
    draw_prompt_button(frame, &prompt.cancel_button_frame, "Cancel", true);
}
