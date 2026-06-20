use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::WorkbenchTextFieldStyle;
use super::metrics::{STEPPER_DIVIDER, STEPPER_GLYPH_SEGMENTS, STEPPER_WIDTH};
use super::segments::push_segments;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_field_stepper(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    style: &WorkbenchTextFieldStyle,
) {
    let left = rect.x + rect.width - STEPPER_WIDTH;
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: left,
            y: rect.y + 4.0,
            width: 1.0,
            height: (rect.height - 8.0).max(1.0),
        },
        Some(clip.clone()),
        order,
        Some(STEPPER_DIVIDER),
        None,
        0.0,
        0.0,
        opacity,
    ));
    let glyph = FrameRect {
        x: left + 4.0,
        y: rect.y + (rect.height - 16.0).max(0.0) * 0.5,
        width: 10.0,
        height: 16.0,
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
