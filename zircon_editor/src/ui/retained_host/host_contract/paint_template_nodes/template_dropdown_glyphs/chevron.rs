use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::WorkbenchDropdownStyle;
use super::metrics::{dropdown_chevron_right, dropdown_chevron_size};
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
    let chevron_size = dropdown_chevron_size();
    let chevron = FrameRect {
        x: rect.x + rect.width - dropdown_chevron_right() - chevron_size,
        y: rect.y + (rect.height - chevron_size).max(0.0) * 0.5,
        width: chevron_size,
        height: chevron_size,
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
