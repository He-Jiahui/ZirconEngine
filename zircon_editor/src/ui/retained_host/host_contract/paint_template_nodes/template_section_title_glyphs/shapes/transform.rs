use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::segments::{self, SectionTitleGlyphSegmentSpec};

const TRANSFORM_SEGMENTS: &[SectionTitleGlyphSegmentSpec] = &[
    SectionTitleGlyphSegmentSpec::new(6, 1, 2, 12),
    SectionTitleGlyphSegmentSpec::new(1, 6, 12, 2),
    SectionTitleGlyphSegmentSpec::new(3, 3, 2, 2),
    SectionTitleGlyphSegmentSpec::new(9, 9, 2, 2),
];

pub(super) fn push_transform_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    segments::push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        TRANSFORM_SEGMENTS,
    );
}
