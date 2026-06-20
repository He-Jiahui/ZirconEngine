use super::super::{ChromeCommand, ChromeCommandKind};
use crate::ui::retained_host::host_contract::paint_frame::HostRgbaFrame;
use crate::ui::retained_host::host_contract::paint_text::draw_text_with_size_and_style;

mod images;
mod shapes;

use images::paint_image_command;
use shapes::{paint_border_command, paint_quad_command};

pub(super) fn paint_chrome_command(frame: &mut HostRgbaFrame, command: &ChromeCommand) {
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
        ChromeCommandKind::Image { payload } => paint_image_command(frame, command, payload),
        ChromeCommandKind::Clip => {}
    }
}
