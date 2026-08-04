use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::data::FrameRect;
use super::super::paint_frame::HostRgbaFrame;
use super::layout_policy::HostTextLayoutPolicy;

mod clip_rect;
mod entry;
mod glyphs;
mod layout;
mod metrics;
mod placement;
mod recording;

use self::clip_rect::resolve_text_pixel_clip;
use self::glyphs::draw_layout_glyphs;
use self::layout::layout_text_run_with_layout_policy;
use self::metrics::clamped_text_metrics;
use self::recording::record_text_run;

pub(in crate::ui::retained_host::host_contract) use entry::{
    draw_text, draw_text_with_size_and_style, draw_text_with_size_and_style_and_layout_policy,
};
#[cfg(test)]
pub(super) use entry::{DEFAULT_FONT_SIZE, DEFAULT_LINE_HEIGHT};

fn draw_text_with_size_and_style_impl(
    frame: &mut HostRgbaFrame,
    rect: FrameRect,
    text: &str,
    clip: Option<&FrameRect>,
    color: [u8; 4],
    font_size: f32,
    line_height: f32,
    style: UiTextRunPaintStyle,
    layout_policy: HostTextLayoutPolicy,
) {
    if text.trim().is_empty() || color[3] == 0 {
        return;
    }
    let Some((clip, effective_clip)) = resolve_text_pixel_clip(frame, &rect, clip) else {
        return;
    };

    let (font_size, line_height) = clamped_text_metrics(rect.height, font_size, line_height);
    let layout = layout_text_run_with_layout_policy(
        &rect,
        text,
        font_size,
        line_height,
        style,
        layout_policy,
    );
    if frame.is_recording() {
        record_text_run(
            frame,
            &clip,
            effective_clip,
            layout.display_text.as_str(),
            color,
            font_size,
            line_height,
            style,
        );
        if frame.record_only() {
            return;
        }
    }
    draw_layout_glyphs(frame, &clip, layout.font_face, &layout.glyphs, color, style);
}
