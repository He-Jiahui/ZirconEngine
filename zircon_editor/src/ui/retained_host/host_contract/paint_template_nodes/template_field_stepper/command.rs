use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::WorkbenchTextFieldStyle;
use super::metrics::{workbench_field_stepper_metrics, STEPPER_GLYPH_SEGMENTS};
use super::segments::push_segments;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_field_stepper(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    style: &WorkbenchTextFieldStyle,
) {
    let metrics = workbench_field_stepper_metrics();
    let left = rect.x + rect.width - metrics.width;
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: left,
            y: rect.y + metrics.divider_inset_y,
            width: metrics.divider_width,
            height: (rect.height - metrics.divider_inset_y * 2.0).max(1.0),
        },
        Some(clip.clone()),
        order,
        Some(style.stepper_divider),
        None,
        0.0,
        0.0,
        opacity,
    ));
    let glyph = FrameRect {
        x: left + metrics.glyph_left_inset,
        y: rect.y + (rect.height - metrics.glyph_height).max(0.0) * 0.5,
        width: metrics.glyph_width,
        height: metrics.glyph_height,
    };
    push_segments(
        commands,
        &glyph,
        clip,
        order + 1,
        style.stepper,
        opacity,
        STEPPER_GLYPH_SEGMENTS,
    );
}
