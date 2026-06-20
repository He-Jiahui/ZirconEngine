use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::metrics::{CHIP_CHEVRON_RIGHT, CHIP_CHEVRON_SIZE};
use super::segments::push_segments;

const CHIP_CHEVRON_SEGMENTS: &[(f32, f32, f32, f32)] = &[
    (3.0, 4.0, 2.0, 2.0),
    (5.0, 6.0, 2.0, 2.0),
    (7.0, 4.0, 2.0, 2.0),
];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_chip_chevron(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    let chevron = FrameRect {
        x: rect.x + rect.width - CHIP_CHEVRON_RIGHT - CHIP_CHEVRON_SIZE,
        y: rect.y + (rect.height - CHIP_CHEVRON_SIZE).max(0.0) * 0.5,
        width: CHIP_CHEVRON_SIZE,
        height: CHIP_CHEVRON_SIZE,
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
