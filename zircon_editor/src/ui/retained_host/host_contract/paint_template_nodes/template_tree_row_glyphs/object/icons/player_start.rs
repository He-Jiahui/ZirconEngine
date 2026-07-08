use super::super::super::super::super::data::FrameRect;
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::super::segments::{
    push_segments, GlyphSegmentSpec, TREE_OBJECT_GLYPH_GRID_UNITS,
};

const PLAYER_START_SEGMENTS: [GlyphSegmentSpec; 5] = [
    GlyphSegmentSpec::new(6, 1, 2, 3),
    GlyphSegmentSpec::new(3, 4, 8, 2),
    GlyphSegmentSpec::new(2, 7, 4, 4),
    GlyphSegmentSpec::new(8, 7, 4, 4),
    GlyphSegmentSpec::new(5, 11, 4, 2),
];

pub(in super::super) fn push_player_start_icon(
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
        &PLAYER_START_SEGMENTS,
    );
}
