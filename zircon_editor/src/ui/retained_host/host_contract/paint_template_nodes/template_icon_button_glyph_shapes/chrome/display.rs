use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_button_glyph_segments::{
    icon_button_segment as seg, push_icon_button_glyph_segments as push_segments,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_grid_icon(
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
            seg(60, 60, 80, 80),
            seg(180, 60, 80, 80),
            seg(60, 180, 80, 80),
            seg(180, 180, 80, 80),
        ],
    );
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_sun_icon(
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
            seg(120, 120, 80, 80),
            seg(144, 40, 32, 48),
            seg(144, 232, 32, 48),
            seg(40, 144, 48, 32),
            seg(232, 144, 48, 32),
        ],
    );
}
