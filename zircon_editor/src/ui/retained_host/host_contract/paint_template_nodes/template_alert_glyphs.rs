use super::super::data::FrameRect;
use super::render_commands::HostPaintCommand;
use super::style_selector::WorkbenchAlertTone as AlertTone;

pub(super) const ALERT_ICON_SIZE: f32 = 18.0;

const ALERT_GLYPH_DARK: [u8; 4] = [8, 18, 18, 255];

pub(super) fn push_alert_mark(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    tone: AlertTone,
    color: [u8; 4],
    opacity: f32,
) {
    match tone {
        AlertTone::Info => {
            commands.push(HostPaintCommand::quad(
                rect.clone(),
                Some(clip.clone()),
                order,
                Some(color),
                None,
                0.0,
                rect.height * 0.5,
                opacity,
            ));
            push_segments(
                commands,
                rect,
                clip,
                order + 1,
                ALERT_GLYPH_DARK,
                opacity,
                &[(8.0, 4.0, 2.0, 2.0), (8.0, 8.0, 2.0, 6.0)],
            );
        }
        AlertTone::Success => {
            commands.push(HostPaintCommand::quad(
                rect.clone(),
                Some(clip.clone()),
                order,
                Some(color),
                None,
                0.0,
                rect.height * 0.5,
                opacity,
            ));
            push_segments(
                commands,
                rect,
                clip,
                order + 1,
                ALERT_GLYPH_DARK,
                opacity,
                &[
                    (4.0, 9.0, 3.0, 2.0),
                    (6.0, 11.0, 3.0, 2.0),
                    (9.0, 6.0, 3.0, 7.0),
                ],
            );
        }
        AlertTone::Warning => {
            let center_x = rect.x + rect.width * 0.5;
            for (row, width) in [3.0, 5.0, 7.0, 9.0, 11.0, 13.0].into_iter().enumerate() {
                commands.push(HostPaintCommand::quad(
                    FrameRect {
                        x: center_x - width * 0.5,
                        y: rect.y + 3.0 + row as f32 * 1.85,
                        width,
                        height: 2.0,
                    },
                    Some(clip.clone()),
                    order,
                    Some(color),
                    None,
                    0.0,
                    1.0,
                    opacity,
                ));
            }
            push_segments(
                commands,
                rect,
                clip,
                order + 1,
                ALERT_GLYPH_DARK,
                opacity,
                &[(8.0, 8.0, 2.0, 4.0), (8.0, 14.0, 2.0, 2.0)],
            );
        }
        AlertTone::Error => {
            commands.push(HostPaintCommand::quad(
                rect.clone(),
                Some(clip.clone()),
                order,
                Some(color),
                None,
                0.0,
                rect.height * 0.5,
                opacity,
            ));
            push_close_mark(commands, rect, clip, order + 1, ALERT_GLYPH_DARK, opacity);
        }
    }
}

pub(super) fn push_close_mark(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[
            (4.0, 4.0, 2.0, 2.0),
            (6.0, 6.0, 2.0, 2.0),
            (8.0, 8.0, 2.0, 2.0),
            (10.0, 10.0, 2.0, 2.0),
            (10.0, 4.0, 2.0, 2.0),
            (8.0, 6.0, 2.0, 2.0),
            (6.0, 8.0, 2.0, 2.0),
            (4.0, 10.0, 2.0, 2.0),
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
    let scale_x = origin.width / ALERT_ICON_SIZE;
    let scale_y = origin.height / ALERT_ICON_SIZE;
    FrameRect {
        x: origin.x + x * scale_x,
        y: origin.y + y * scale_y,
        width: (width * scale_x).max(1.0),
        height: (height * scale_y).max(1.0),
    }
}
