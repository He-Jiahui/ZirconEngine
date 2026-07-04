use super::super::data::FrameRect;
use super::super::paint_frame::HostRgbaFrame;
use super::super::paint_geometry::is_visible_frame;
use super::super::paint_text::{draw_text, measure_runtime_text_width};

const TEXT_BAR_FONT_SIZE: f32 = 12.0;
const TEXT_BAR_LINE_HEIGHT: f32 = 16.0;

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
    draw_text(frame, text_bars_frame(x, y, text), text, clip, color);
}

fn text_bars_frame(x: f32, y: f32, text: &str) -> FrameRect {
    FrameRect {
        x,
        y,
        width: measure_runtime_text_width(text, TEXT_BAR_FONT_SIZE).max(1.0),
        height: TEXT_BAR_LINE_HEIGHT,
    }
}

#[cfg(test)]
mod tests {
    use super::text_bars_frame;

    #[test]
    fn text_bars_frame_uses_runtime_text_measurement() {
        let narrow = text_bars_frame(0.0, 0.0, "iiiiiiiiiiii");
        let wide = text_bars_frame(0.0, 0.0, "WWWWWWWWWWWW");

        assert!(
            wide.width > narrow.width + 8.0,
            "same-character-count text bars should follow runtime glyph width, narrow={narrow:?}, wide={wide:?}"
        );
    }
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
