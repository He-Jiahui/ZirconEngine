use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_button_glyph_segments::{
    icon_button_segment as seg, push_icon_button_glyph_segments as push_segments,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_move_icon(
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
            seg(144, 40, 32, 240),
            seg(40, 144, 240, 32),
            seg(120, 60, 80, 24),
            seg(120, 240, 80, 24),
            seg(60, 120, 24, 80),
            seg(240, 120, 24, 80),
        ],
    );
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_rotate_icon(
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
            seg(80, 60, 120, 26),
            seg(60, 80, 26, 100),
            seg(80, 210, 140, 26),
            seg(220, 140, 26, 90),
            seg(180, 40, 70, 26),
            seg(220, 40, 26, 70),
        ],
    );
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_scale_icon(
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
            seg(60, 60, 100, 26),
            seg(60, 60, 26, 100),
            seg(160, 160, 100, 26),
            seg(240, 160, 26, 100),
            seg(80, 220, 160, 26),
            seg(200, 100, 26, 140),
        ],
    );
}
