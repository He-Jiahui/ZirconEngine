use super::super::super::super::super::data::FrameRect;
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::super::segments::{
    push_segments, GlyphSegmentSpec, TREE_OBJECT_GLYPH_GRID_UNITS,
};

const AUDIO_SEGMENTS: [GlyphSegmentSpec; 6] = [
    GlyphSegmentSpec::new(2, 5, 3, 4),
    GlyphSegmentSpec::new(5, 3, 2, 8),
    GlyphSegmentSpec::new(8, 4, 1, 2),
    GlyphSegmentSpec::new(10, 3, 1, 4),
    GlyphSegmentSpec::new(8, 8, 1, 2),
    GlyphSegmentSpec::new(10, 7, 1, 4),
];

pub(in super::super) fn push_audio_icon(
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
        &AUDIO_SEGMENTS,
    );
}
