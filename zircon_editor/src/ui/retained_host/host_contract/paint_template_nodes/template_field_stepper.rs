use super::super::data::FrameRect;
use super::render_commands::HostPaintCommand;
use super::style_selector::WorkbenchTextFieldStyle;

pub(super) const STEPPER_WIDTH: f32 = 18.0;
pub(super) const STEPPER_DIVIDER: [u8; 4] = [42, 53, 60, 255];

pub(super) fn push_field_stepper(
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
        &[
            (4.0, 2.0, 2.0, 2.0),
            (2.0, 4.0, 6.0, 1.4),
            (2.0, 11.0, 6.0, 1.4),
            (4.0, 13.0, 2.0, 2.0),
        ],
    );
}

fn push_segments(
    commands: &mut Vec<HostPaintCommand>,
    origin: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
    segments: &[(f32, f32, f32, f32)],
) {
    for (x, y, width, height) in segments {
        commands.push(HostPaintCommand::quad(
            scaled_rect(origin, *x, *y, *width, *height),
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            1.0,
            opacity,
        ));
    }
}

fn scaled_rect(origin: &FrameRect, x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    let scale_x = origin.width / 10.0;
    let scale_y = origin.height / 16.0;
    FrameRect {
        x: origin.x + x * scale_x,
        y: origin.y + y * scale_y,
        width: (width * scale_x).max(1.0),
        height: (height * scale_y).max(1.0),
    }
}
