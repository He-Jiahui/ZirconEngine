use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::style_selector::WorkbenchAlertTone as AlertTone;
use super::super::close::push_close_mark;
use super::super::segments::alert_segment as seg;
use super::palette::alert_glyph_dark;
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
    let glyph_dark = alert_glyph_dark();
    match tone {
        AlertTone::Info => push_round_mark(
            commands,
            rect,
            clip,
            order,
            color,
            opacity,
            &[seg(8, 4, 2, 2), seg(8, 8, 2, 6)],
            glyph_dark,
        ),
        AlertTone::Success => push_round_mark(
            commands,
            rect,
            clip,
            order,
            color,
            opacity,
            &[seg(4, 9, 3, 2), seg(6, 11, 3, 2), seg(9, 6, 3, 7)],
            glyph_dark,
        ),
        AlertTone::Warning => {
            push_warning_mark(commands, rect, clip, order, color, opacity, glyph_dark)
        }
        AlertTone::Error => {
            push_round_surface(commands, rect, clip, order, color, opacity);
            push_close_mark(commands, rect, clip, order + 1, glyph_dark, opacity);
        }
    }
}
