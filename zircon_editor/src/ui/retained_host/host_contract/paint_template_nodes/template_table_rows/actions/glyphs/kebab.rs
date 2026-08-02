use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;

use super::segments::{GlyphSegmentSpec, push_segments};

const KEBAB_SEGMENTS: [GlyphSegmentSpec; 3] = [
    GlyphSegmentSpec::new(6, 3, 2, 2),
    GlyphSegmentSpec::new(6, 6, 2, 2),
    GlyphSegmentSpec::new(6, 9, 2, 2),
];

pub(in super::super) fn push_table_kebab(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(commands, rect, &KEBAB_SEGMENTS, clip, order, color, opacity);
}
