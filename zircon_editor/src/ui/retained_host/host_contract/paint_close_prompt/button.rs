use super::super::data::FrameRect;
use super::super::paint_frame::HostRgbaFrame;
use super::super::paint_primitives::{draw_border, draw_rect, draw_text_bars_clipped};
use super::colors::{ACCENT, BUTTON, BUTTON_DISABLED, MUTED, TEXT};

pub(in crate::ui::retained_host::host_contract) fn draw_prompt_button(
    frame: &mut HostRgbaFrame,
    button: &FrameRect,
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
