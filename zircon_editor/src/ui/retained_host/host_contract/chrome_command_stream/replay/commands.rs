use super::super::{ChromeCommand, ChromeCommandKind, ChromeCommandStream};
use crate::ui::retained_host::host_contract::paint_frame::HostRgbaFrame;
use crate::ui::retained_host::host_contract::paint_text::draw_text_with_size_and_style;

mod images;
mod shapes;

use images::paint_image_command;
use shapes::{paint_border_command, paint_quad_command, paint_rounded_box_commands};

pub(super) fn paint_chrome_command_pair(
    frame: &mut HostRgbaFrame,
    fill: &ChromeCommand,
    border: &ChromeCommand,
) -> bool {
    let (
        ChromeCommandKind::Quad {
            color: fill_color,
            corner_radius,
        },
        ChromeCommandKind::Border {
            color: border_color,
            width,
            corner_radius: border_radius,
        },
    ) = (&fill.kind, &border.kind)
    else {
        return false;
    };
    if fill_color[3] == 0
        || border_color[3] == 0
        || !width.is_finite()
        || *width <= 0.0
        || fill.frame != border.frame
        || fill.clip != border.clip
        || corner_radius.to_bits() != border_radius.to_bits()
    {
        return false;
    }
    paint_rounded_box_commands(
        frame,
        fill,
        *fill_color,
        *border_color,
        *width,
        *corner_radius,
    );
    true
}

pub(super) fn paint_chrome_command(
    frame: &mut HostRgbaFrame,
    stream: &ChromeCommandStream,
    command: &ChromeCommand,
) {
    match &command.kind {
        ChromeCommandKind::Quad {
            color,
            corner_radius,
        } => paint_quad_command(frame, command, *color, *corner_radius),
        ChromeCommandKind::Border {
            color,
            width,
            corner_radius,
        } => paint_border_command(frame, command, *color, *width, *corner_radius),
        ChromeCommandKind::Text {
            text,
            color,
            size,
            line_height,
            style,
        } => draw_text_with_size_and_style(
            frame,
            command.frame.clone(),
            text,
            command.clip.as_ref(),
            *color,
            *size,
            *line_height,
            *style,
        ),
        ChromeCommandKind::Image { payload } => {
            paint_image_command(frame, stream, command, payload)
        }
        ChromeCommandKind::Clip => {}
    }
}
