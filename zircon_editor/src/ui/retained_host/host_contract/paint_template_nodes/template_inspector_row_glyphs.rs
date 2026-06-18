use super::super::data::FrameRect;
use super::super::paint_theme::PALETTE;
use super::render_commands::HostPaintCommand;

const INSPECTOR_SWATCH_SIZE: f32 = 12.0;
const MATERIAL_SWATCH: [u8; 4] = [34, 176, 192, 255];
const MATERIAL_SWATCH_BORDER: [u8; 4] = [21, 95, 105, 255];

pub(super) fn push_inspector_swatch(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let swatch = FrameRect {
        x: rect.x + (rect.width - INSPECTOR_SWATCH_SIZE).max(0.0) * 0.5,
        y: rect.y + (rect.height - INSPECTOR_SWATCH_SIZE).max(0.0) * 0.5,
        width: INSPECTOR_SWATCH_SIZE,
        height: INSPECTOR_SWATCH_SIZE,
    };
    commands.push(HostPaintCommand::quad(
        swatch,
        Some(clip.clone()),
        order,
        Some(MATERIAL_SWATCH),
        Some(MATERIAL_SWATCH_BORDER),
        1.0,
        INSPECTOR_SWATCH_SIZE * 0.5,
        opacity,
    ));
}

pub(super) fn push_inspector_cube_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_inspector_segments(
        commands,
        clip,
        order,
        color,
        opacity,
        &[
            FrameRect {
                x: rect.x + 3.0,
                y: rect.y + 3.0,
                width: rect.width - 6.0,
                height: rect.height - 6.0,
            },
            FrameRect {
                x: rect.x + 5.0,
                y: rect.y + 1.0,
                width: rect.width - 6.0,
                height: 2.0,
            },
            FrameRect {
                x: rect.x + rect.width - 3.0,
                y: rect.y + 4.0,
                width: 2.0,
                height: rect.height - 7.0,
            },
        ],
        1.0,
    );
}

pub(super) fn push_inspector_down_chevron(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    let parts = if rect.width >= 14.0 && rect.height >= 14.0 {
        let block = 3.0;
        let center_x = rect.x + rect.width * 0.5;
        let center_y = rect.y + rect.height * 0.5;
        [
            FrameRect {
                x: center_x - block * 1.5,
                y: center_y - block,
                width: block,
                height: block,
            },
            FrameRect {
                x: center_x - block * 0.5,
                y: center_y,
                width: block,
                height: block,
            },
            FrameRect {
                x: center_x + block * 0.5,
                y: center_y - block,
                width: block,
                height: block,
            },
        ]
    } else {
        [
            FrameRect {
                x: rect.x + 2.0,
                y: rect.y + 3.0,
                width: 2.0,
                height: 2.0,
            },
            FrameRect {
                x: rect.x + 4.0,
                y: rect.y + 5.0,
                width: 2.0,
                height: 2.0,
            },
            FrameRect {
                x: rect.x + 6.0,
                y: rect.y + 3.0,
                width: 2.0,
                height: 2.0,
            },
        ]
    };
    push_inspector_segments(commands, clip, order, color, opacity, &parts, 0.0);
}

pub(super) fn push_inspector_check_tick(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_inspector_segments(
        commands,
        clip,
        order,
        PALETTE.shell_background,
        opacity,
        &[
            FrameRect {
                x: rect.x + 3.0,
                y: rect.y + 7.0,
                width: 3.0,
                height: 3.0,
            },
            FrameRect {
                x: rect.x + 5.0,
                y: rect.y + 9.0,
                width: 3.0,
                height: 3.0,
            },
            FrameRect {
                x: rect.x + 8.0,
                y: rect.y + 4.0,
                width: 3.0,
                height: 8.0,
            },
        ],
        1.0,
    );
}

fn push_inspector_segments(
    commands: &mut Vec<HostPaintCommand>,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
    segments: &[FrameRect],
    radius: f32,
) {
    for part in segments {
        commands.push(HostPaintCommand::quad(
            part.clone(),
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            radius,
            opacity,
        ));
    }
}
