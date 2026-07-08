use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::data::FrameRect;
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::PixelRect;

pub(super) fn record_text_run(
    frame: &mut HostRgbaFrame,
    clip: &PixelRect,
    effective_clip: Option<FrameRect>,
    text: &str,
    color: [u8; 4],
    font_size: f32,
    line_height: f32,
    style: UiTextRunPaintStyle,
) {
    frame.record_text(
        FrameRect {
            x: clip.x0 as f32,
            y: clip.y0 as f32,
            width: clip.x1.saturating_sub(clip.x0) as f32,
            height: clip.y1.saturating_sub(clip.y0) as f32,
        },
        effective_clip,
        text,
        color,
        font_size,
        line_height,
        style,
    );
}
