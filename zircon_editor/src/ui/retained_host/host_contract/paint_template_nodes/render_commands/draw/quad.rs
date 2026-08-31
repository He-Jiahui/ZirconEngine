use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_primitives::{
    draw_rect_clipped, draw_rounded_border_clipped, draw_rounded_box_clipped,
    draw_rounded_rect_clipped,
};
use super::super::command::HostPaintCommand;
use super::{border::draw_border_width, color::color_with_opacity};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn draw_quad_command(
    frame: &mut HostRgbaFrame,
    command: &HostPaintCommand,
) -> bool {
    let clip = command.clip_frame.as_ref();
    let background_color = command
        .background_color
        .map(|color| color_with_opacity(color, command.opacity));
    let border_color = command
        .border_color
        .map(|color| color_with_opacity(color, command.opacity));
    if command.border_width.is_finite() && command.border_width > 0.0 {
        if let (Some(fill_color), Some(border_color)) = (background_color, border_color) {
            if fill_color[3] != 0 && border_color[3] != 0 {
                draw_rounded_box_clipped(
                    frame,
                    command.frame.clone(),
                    clip,
                    fill_color,
                    border_color,
                    command.border_width,
                    command.corner_radius,
                );
                return true;
            }
        }
    }
    let mut drew_any = false;
    if let Some(color) = background_color {
        if command.corner_radius > 0.0 {
            draw_rounded_rect_clipped(
                frame,
                command.frame.clone(),
                clip,
                color,
                command.corner_radius,
            );
        } else {
            draw_rect_clipped(frame, command.frame.clone(), clip, color);
        }
        drew_any = true;
    }
    if command.border_width > 0.0 {
        if let Some(color) = border_color {
            if command.corner_radius > 0.0 {
                draw_rounded_border_clipped(
                    frame,
                    command.frame.clone(),
                    clip,
                    color,
                    command.border_width,
                    command.corner_radius,
                );
            } else {
                draw_border_width(frame, &command.frame, clip, color, command.border_width);
            }
            drew_any = true;
        }
    }
    drew_any
}
