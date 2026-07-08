use super::super::super::super::super::data::FrameRect;
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::super::segments::{
    push_segments, GlyphSegmentSpec, TREE_OBJECT_GLYPH_GRID_UNITS,
};

const CUBE_SEGMENTS: [GlyphSegmentSpec; 7] = [
    GlyphSegmentSpec::new(3, 2, 8, 1),
    GlyphSegmentSpec::new(2, 3, 1, 7),
    GlyphSegmentSpec::new(11, 3, 1, 7),
    GlyphSegmentSpec::new(3, 10, 8, 1),
    GlyphSegmentSpec::new(6, 0, 1, 3),
    GlyphSegmentSpec::new(6, 10, 1, 3),
    GlyphSegmentSpec::new(2, 6, 10, 1),
];

pub(in super::super) fn push_cube_icon(
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
        TREE_OBJECT_GLYPH_GRID_UNITS,
        clip,
        order,
        color,
        opacity,
        &CUBE_SEGMENTS,
    );
}
