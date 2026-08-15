use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::segments::{push_segments, GlyphSegmentSpec};

const RIGHT_CHEVRON_SEGMENTS: [GlyphSegmentSpec; 3] = [
    GlyphSegmentSpec::new(5, 3, 2, 3),
    GlyphSegmentSpec::new(7, 6, 2, 2),
    GlyphSegmentSpec::new(5, 8, 2, 3),
];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_right_chevron(
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
        &RIGHT_CHEVRON_SEGMENTS,
        clip,
        order,
        color,
        opacity,
    );
}
