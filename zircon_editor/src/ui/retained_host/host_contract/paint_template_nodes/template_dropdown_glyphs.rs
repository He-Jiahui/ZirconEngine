use super::super::data::FrameRect;
use super::render_commands::HostPaintCommand;
use super::style_selector::WorkbenchDropdownStyle;

const DROPDOWN_CHEVRON_SIZE: f32 = 14.0;
const DROPDOWN_CHEVRON_RIGHT: f32 = 7.0;
pub(super) const DROPDOWN_CHEVRON_RESERVE: f32 =
    DROPDOWN_CHEVRON_SIZE + DROPDOWN_CHEVRON_RIGHT + 4.0;

pub(super) fn push_dropdown_chevron(
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
        &[
            (3.0, 5.0, 2.0, 2.0),
            (5.0, 7.0, 2.0, 2.0),
            (7.0, 5.0, 2.0, 2.0),
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
    let scale_x = origin.width / 14.0;
    let scale_y = origin.height / 14.0;
    FrameRect {
        x: origin.x + x * scale_x,
        y: origin.y + y * scale_y,
        width: (width * scale_x).max(1.0),
        height: (height * scale_y).max(1.0),
    }
}
