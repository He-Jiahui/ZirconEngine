use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::style_selector::WorkbenchAlertTone as AlertTone;
use super::super::close::push_close_mark;
use super::palette::ALERT_GLYPH_DARK;
use super::round::{push_round_mark, push_round_surface};
use super::warning::push_warning_mark;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_alert_mark(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    tone: AlertTone,
    color: [u8; 4],
    opacity: f32,
) {
    match tone {
        AlertTone::Info => push_round_mark(
            commands,
            rect,
            clip,
            order,
            color,
            opacity,
            &[(8.0, 4.0, 2.0, 2.0), (8.0, 8.0, 2.0, 6.0)],
        ),
        AlertTone::Success => push_round_mark(
            commands,
            rect,
            clip,
            order,
            color,
            opacity,
            &[
                (4.0, 9.0, 3.0, 2.0),
                (6.0, 11.0, 3.0, 2.0),
                (9.0, 6.0, 3.0, 7.0),
            ],
        ),
        AlertTone::Warning => push_warning_mark(commands, rect, clip, order, color, opacity),
        AlertTone::Error => {
            push_round_surface(commands, rect, clip, order, color, opacity);
            push_close_mark(commands, rect, clip, order + 1, ALERT_GLYPH_DARK, opacity);
        }
    }
}
