use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_button_glyph_segments::{
    icon_button_segment as seg, push_icon_button_glyph_segments as push_segments,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_chevron_down_icon(
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
            seg(80, 120, 40, 40),
            seg(120, 160, 80, 40),
            seg(200, 120, 40, 40),
        ],
    );
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_more_icon(
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
            seg(140, 60, 40, 40),
            seg(140, 140, 40, 40),
            seg(140, 220, 40, 40),
        ],
    );
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_close_icon(
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
            seg(80, 60, 40, 40),
            seg(120, 100, 40, 40),
            seg(160, 140, 40, 40),
            seg(200, 180, 40, 40),
            seg(200, 60, 40, 40),
            seg(160, 100, 40, 40),
            seg(120, 140, 40, 40),
            seg(80, 180, 40, 40),
        ],
    );
}
