use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_geometry::is_visible_frame;
use super::super::super::evidence::PaintNodeOwnerScope;
use super::super::command::{HostPaintCommand, HostPaintCommandKind};
use super::{image::draw_image_command, quad::draw_quad_command, text::draw_text_command};

/// 分派只消费一条排好层级的宿主命令；图片、文字和表面各走对应的录制或即时绘制入口。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn draw_host_paint_command(
    frame: &mut HostRgbaFrame,
    command: &HostPaintCommand,
) -> bool {
    let _owner = PaintNodeOwnerScope::enter(command.owner.as_ref());
    if command.opacity <= 0.0 || !command.opacity.is_finite() || !is_visible_frame(&command.frame) {
        return false;
    }

    let draw = |frame: &mut HostRgbaFrame| {
        frame.with_render_source_command(command.source_render_command_ref, |frame| {
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
        })
    };
    if let Some(source) = command.source_surface_frame.as_ref() {
        frame.with_render_source_frame(Some(source), draw)
    } else {
        draw(frame)
    }
}
