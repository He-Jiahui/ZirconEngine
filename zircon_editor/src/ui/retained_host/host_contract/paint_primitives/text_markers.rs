use super::super::data::FrameRect;
use super::super::paint_frame::HostRgbaFrame;
use super::super::paint_geometry::is_visible_frame;
use super::super::paint_text::draw_text;

pub(in crate::ui::retained_host::host_contract) fn draw_text_bars(
    frame: &mut HostRgbaFrame,
    x: f32,
    y: f32,
    text: &str,
    color: [u8; 4],
) {
    draw_text_bars_clipped(frame, x, y, text, None, color);
}

pub(in crate::ui::retained_host::host_contract) fn draw_text_bars_clipped(
    frame: &mut HostRgbaFrame,
    x: f32,
    y: f32,
    text: &str,
    clip: Option<&FrameRect>,
    color: [u8; 4],
) {
    draw_text(
        frame,
        FrameRect {
            x,
            y,
            width: (text.chars().count() as f32 * 8.0).max(1.0),
            height: 16.0,
        },
        text,
        clip,
        color,
    );
}

pub(in crate::ui::retained_host::host_contract) fn draw_label_marker(
    frame: &mut HostRgbaFrame,
    target: &FrameRect,
    label: &str,
    color: [u8; 4],
) {
    if !is_visible_frame(target) {
        return;
    }
    draw_text(
        frame,
        FrameRect {
            x: target.x + 12.0,
            y: target.y + ((target.height - 16.0).max(0.0) * 0.5),
            width: (target.width - 24.0).max(1.0),
            height: target.height.min(18.0).max(1.0),
        },
        label,
        Some(target),
        color,
    );
}
