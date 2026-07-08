use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_button_glyph_segments::{
    icon_button_segment as seg, push_icon_button_glyph_segments as push_segments,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_image_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[
            seg(60, 60, 200, 24),
            seg(60, 80, 24, 180),
            seg(240, 80, 24, 180),
            seg(80, 240, 160, 24),
            seg(100, 200, 60, 24),
            seg(140, 160, 60, 24),
            seg(200, 120, 32, 32),
        ],
    );
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_audio_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[
            seg(60, 120, 60, 80),
            seg(120, 80, 40, 160),
            seg(180, 100, 24, 40),
            seg(220, 80, 24, 80),
            seg(180, 180, 24, 40),
            seg(220, 160, 24, 80),
        ],
    );
}
