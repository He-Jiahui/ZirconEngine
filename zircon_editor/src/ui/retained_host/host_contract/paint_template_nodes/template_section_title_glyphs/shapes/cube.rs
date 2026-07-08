use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::segments::{self, SectionTitleGlyphSegmentSpec};

const CUBE_SEGMENTS: &[SectionTitleGlyphSegmentSpec] = &[
    SectionTitleGlyphSegmentSpec::new(5, 1, 4, 2),
    SectionTitleGlyphSegmentSpec::new(3, 3, 2, 7),
    SectionTitleGlyphSegmentSpec::new(9, 3, 2, 7),
    SectionTitleGlyphSegmentSpec::new(5, 11, 4, 2),
    SectionTitleGlyphSegmentSpec::new(1, 5, 2, 4),
    SectionTitleGlyphSegmentSpec::new(11, 5, 2, 4),
];

pub(super) fn push_cube_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    segments::push_segments(commands, rect, clip, order, color, opacity, CUBE_SEGMENTS);
}
