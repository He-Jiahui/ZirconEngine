use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::segments::{self, SectionTitleGlyphSegmentSpec};

const MESH_SEGMENTS: &[SectionTitleGlyphSegmentSpec] = &[
    SectionTitleGlyphSegmentSpec::new(2, 2, 10, 2),
    SectionTitleGlyphSegmentSpec::new(2, 6, 10, 2),
    SectionTitleGlyphSegmentSpec::new(2, 10, 10, 2),
    SectionTitleGlyphSegmentSpec::new(2, 2, 2, 10),
    SectionTitleGlyphSegmentSpec::new(6, 2, 2, 10),
    SectionTitleGlyphSegmentSpec::new(10, 2, 2, 10),
];

pub(super) fn push_mesh_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    segments::push_segments(commands, rect, clip, order, color, opacity, MESH_SEGMENTS);
}
