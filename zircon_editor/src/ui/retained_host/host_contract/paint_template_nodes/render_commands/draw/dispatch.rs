use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_geometry::is_visible_frame;
use super::super::command::{HostPaintCommand, HostPaintCommandKind};
use super::{image::draw_image_command, quad::draw_quad_command, text::draw_text_command};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn draw_host_paint_command(
    frame: &mut HostRgbaFrame,
    command: &HostPaintCommand,
) -> bool {
    if command.opacity <= 0.0 || !command.opacity.is_finite() || !is_visible_frame(&command.frame) {
        return false;
    }

    match command.kind {
        HostPaintCommandKind::Group => false,
        HostPaintCommandKind::Quad => {
            zircon_runtime::profile_scope!("editor", "host_painter", "paint_command_quad");
            draw_quad_command(frame, command)
        }
        HostPaintCommandKind::Text => {
            zircon_runtime::profile_scope!("editor", "host_painter", "paint_command_text");
            draw_text_command(frame, command)
        }
        HostPaintCommandKind::Image => {
            zircon_runtime::profile_scope!("editor", "host_painter", "paint_command_image");
            draw_image_command(frame, command)
        }
    }
}
