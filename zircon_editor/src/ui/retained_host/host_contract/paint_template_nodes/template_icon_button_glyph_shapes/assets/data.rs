use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_button_glyph_segments::{
    icon_button_segment as seg, push_icon_button_glyph_segments as push_segments,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_cube_icon(
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
            seg(80, 60, 160, 24),
            seg(60, 80, 24, 140),
            seg(240, 80, 24, 140),
            seg(80, 220, 160, 24),
            seg(150, 40, 24, 200),
            seg(60, 140, 200, 24),
        ],
    );
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_graph_icon(
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
            seg(80, 80, 60, 60),
            seg(200, 60, 60, 60),
            seg(180, 200, 60, 60),
            seg(120, 100, 100, 24),
            seg(200, 120, 24, 100),
        ],
    );
}
