use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::WorkbenchDropdownStyle;
use super::metrics::{DROPDOWN_CHEVRON_RIGHT, DROPDOWN_CHEVRON_SIZE};
use super::segments::push_segments;

const DROPDOWN_CHEVRON_SEGMENTS: &[(f32, f32, f32, f32)] = &[
    (3.0, 5.0, 2.0, 2.0),
    (5.0, 7.0, 2.0, 2.0),
    (7.0, 5.0, 2.0, 2.0),
];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_dropdown_chevron(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    style: &WorkbenchDropdownStyle,
) {
    let chevron = FrameRect {
        x: rect.x + rect.width - DROPDOWN_CHEVRON_RIGHT - DROPDOWN_CHEVRON_SIZE,
        y: rect.y + (rect.height - DROPDOWN_CHEVRON_SIZE).max(0.0) * 0.5,
        width: DROPDOWN_CHEVRON_SIZE,
        height: DROPDOWN_CHEVRON_SIZE,
    };
    push_segments(
        commands,
        &chevron,
        clip,
        order,
        style.chevron,
        opacity,
        DROPDOWN_CHEVRON_SEGMENTS,
    );
}
