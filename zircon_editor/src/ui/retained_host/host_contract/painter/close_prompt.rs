use super::super::data::HostWindowPresentationData;
use super::frame::HostRgbaFrame;
use super::primitives::{draw_border, draw_rect, draw_text_bars_clipped};
use super::theme::PALETTE;

const OVERLAY: [u8; 4] = [3, 5, 9, 168];
const DIALOG: [u8; 4] = PALETTE.surface;
const DIALOG_INSET: [u8; 4] = PALETTE.surface_inset;
const BUTTON: [u8; 4] = PALETTE.surface_hover;
const BUTTON_DISABLED: [u8; 4] = [31, 36, 45, 255];
const TEXT: [u8; 4] = PALETTE.text;
const MUTED: [u8; 4] = PALETTE.text_muted;
const WARNING: [u8; 4] = PALETTE.warning;
const ACCENT: [u8; 4] = PALETTE.focus_ring;

pub(super) fn draw_close_prompt(
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

fn draw_prompt_button(
    frame: &mut HostRgbaFrame,
    button: &super::super::data::FrameRect,
    label: &str,
    enabled: bool,
) {
    draw_rect(
        frame,
        button.clone(),
        if enabled { BUTTON } else { BUTTON_DISABLED },
    );
    draw_border(frame, button.clone(), if enabled { ACCENT } else { MUTED });
    draw_text_bars_clipped(
        frame,
        button.x + 12.0,
        button.y + 8.0,
        label,
        Some(button),
        if enabled { TEXT } else { MUTED },
    );
}

fn prompt_details_frame(
    prompt: &super::super::data::HostClosePromptData,
) -> super::super::data::FrameRect {
    super::super::data::FrameRect {
        x: prompt.dialog_frame.x + 18.0,
        y: prompt.dialog_frame.y + 76.0,
        width: (prompt.dialog_frame.width - 36.0).max(0.0),
        height: 42.0,
    }
}
