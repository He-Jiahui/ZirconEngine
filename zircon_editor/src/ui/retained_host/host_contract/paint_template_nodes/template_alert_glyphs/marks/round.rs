use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::segments::{AlertGlyphSegmentSpec, push_segments};

pub(super) fn push_round_mark(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
    segments: &[AlertGlyphSegmentSpec],
    glyph_dark: [u8; 4],
) {
    push_round_surface(commands, rect, clip, order, color, opacity);
    push_segments(
        commands,
        rect,
        clip,
        order + 1,
        glyph_dark,
        opacity,
        segments,
    );
}

pub(super) fn push_round_surface(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        rect.height * 0.5,
        opacity,
    ));
}
