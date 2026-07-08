use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::metrics::{chip_glyph_chevron_right, chip_glyph_chevron_size};
use super::segments::{push_segments, ChipGlyphSegmentSpec};

const CHIP_CHEVRON_SEGMENTS: &[ChipGlyphSegmentSpec] = &[
    ChipGlyphSegmentSpec::new(3, 4, 2, 2),
    ChipGlyphSegmentSpec::new(5, 6, 2, 2),
    ChipGlyphSegmentSpec::new(7, 4, 2, 2),
];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_chip_chevron(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    let chevron_size = chip_glyph_chevron_size();
    let chevron = FrameRect {
        x: rect.x + rect.width - chip_glyph_chevron_right() - chevron_size,
        y: rect.y + (rect.height - chevron_size).max(0.0) * 0.5,
        width: chevron_size,
        height: chevron_size,
    };
    push_segments(
        commands,
        &chevron,
        clip,
        order,
        color,
        opacity,
        CHIP_CHEVRON_SEGMENTS,
    );
}
