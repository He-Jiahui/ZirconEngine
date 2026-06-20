use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::segments::push_segments;
use super::palette::ALERT_GLYPH_DARK;

pub(super) fn push_warning_mark(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
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
