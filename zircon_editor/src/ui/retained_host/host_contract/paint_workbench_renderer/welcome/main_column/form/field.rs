use super::super::super::super::super::data::FrameRect;
use super::super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::super::paint_primitives::{
    draw_border_clipped, draw_rect_clipped, draw_text_bars_clipped,
};
use super::super::super::super::SEPARATOR;
use super::super::super::style::{WELCOME_MUTED_TEXT, WELCOME_SURFACE, WELCOME_TEXT};

pub(in crate::ui::retained_host::host_contract) fn draw_welcome_field(
    frame: &mut HostRgbaFrame,
    rect: &FrameRect,
    label: &str,
    value: &str,
    clip: &FrameRect,
) {
    draw_rect_clipped(frame, rect.clone(), Some(clip), WELCOME_SURFACE);
    draw_border_clipped(frame, rect.clone(), Some(clip), SEPARATOR);
    draw_text_bars_clipped(
        frame,
        rect.x + 14.0,
        rect.y + 8.0,
        label,
        Some(clip),
        WELCOME_MUTED_TEXT,
    );
    draw_text_bars_clipped(
        frame,
        rect.x + 14.0,
        rect.y + 30.0,
        value,
        Some(clip),
        WELCOME_TEXT,
    );
}
