use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::data::FrameRect;
use super::super::super::paint_frame::HostRgbaFrame;

pub(super) fn record_text_run(
    frame: &mut HostRgbaFrame,
    rect: &FrameRect,
    effective_clip: Option<FrameRect>,
    text: &str,
    color: [u8; 4],
    font_size: f32,
    line_height: f32,
    style: UiTextRunPaintStyle,
) {
    frame.record_text(
        // Clipping limits coverage without changing the fractional layout origin.
        rect.clone(),
        effective_clip,
        text,
        color,
        font_size,
        line_height,
        style,
    );
}
