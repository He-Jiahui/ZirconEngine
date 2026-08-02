use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;

use super::segments::{GlyphSegmentSpec, push_segments};

const GEAR_SEGMENTS: [GlyphSegmentSpec; 5] = [
    GlyphSegmentSpec::new(4, 2, 6, 1),
    GlyphSegmentSpec::new(4, 11, 6, 1),
    GlyphSegmentSpec::new(2, 4, 1, 6),
    GlyphSegmentSpec::new(11, 4, 1, 6),
    GlyphSegmentSpec::new(6, 6, 2, 2),
];

pub(in super::super) fn push_table_gear(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(commands, rect, &GEAR_SEGMENTS, clip, order, color, opacity);
}
