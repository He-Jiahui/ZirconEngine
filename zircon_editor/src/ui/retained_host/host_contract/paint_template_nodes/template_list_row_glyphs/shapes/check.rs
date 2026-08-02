use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::segments::{GlyphSegmentSpec, push_segments};

const CHECK_MARK_SEGMENTS: [GlyphSegmentSpec; 3] = [
    GlyphSegmentSpec::new(2, 7, 3, 2),
    GlyphSegmentSpec::new(4, 9, 3, 2),
    GlyphSegmentSpec::new(7, 4, 3, 7),
];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_check_mark(
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
        &CHECK_MARK_SEGMENTS,
        clip,
        order,
        color,
        opacity,
    );
}
