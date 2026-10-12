use zircon_runtime_interface::ui::surface::UiRenderCommand;

use super::super::data::FrameRect;
use super::super::paint_frame::HostRgbaFrame;
use super::render_command_conversion::{push_runtime_command, runtime_render_commands_to_host};

mod command;
mod draw;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use command::{
    HostPaintCommand, HostPaintCommandKind, PaintNodeIdentity,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use draw::draw_host_paint_commands;

/// Runtime 命令先过宿主转换，再共用模板绘制器的排序和帧录制路径。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn draw_runtime_render_commands(
    frame: &mut HostRgbaFrame,
    commands: &[UiRenderCommand],
    clip_frame: Option<&FrameRect>,
) -> bool {
    let host_commands = runtime_render_commands_to_host(commands, clip_frame);
    draw_host_paint_commands(frame, &host_commands)
}

#[cfg(test)]
pub(crate) fn paint_runtime_render_commands_for_test(
    width: u32,
    height: u32,
    commands: &[UiRenderCommand],
) -> Vec<u8> {
    let mut frame = HostRgbaFrame::filled(width, height, [0, 0, 0, 255]);
    draw_runtime_render_commands(&mut frame, commands, None);
    frame.into_bytes()
}

/// Consume a published source frame through the existing command converter and recorder.
pub(in crate::ui::retained_host::host_contract) fn draw_runtime_surface_frame(
    frame: &mut HostRgbaFrame,
    source: &std::sync::Arc<zircon_runtime_interface::ui::surface::UiSurfaceFrame>,
    origin_x: f32,
    origin_y: f32,
    scale: f32,
    clip: &FrameRect,
) -> bool {
    use zircon_runtime_interface::ui::surface::UiRenderFrameCommandRef;
    let mut host_commands = Vec::with_capacity(source.render_extract.list.commands.len());
    for (index, command) in source.render_extract.list.commands.iter().enumerate() {
        let range = source.render_extract.command_range(command.node_id);
        let command_ref = range.and_then(|range| {
            (range.contains(&index)).then(|| {
                UiRenderFrameCommandRef::new(command.node_id, (index - range.start) as u32)
            })
        });
        let start = host_commands.len();
        push_runtime_command(&mut host_commands, command, None);
        for projected in &mut host_commands[start..] {
            projected.frame.x = origin_x + projected.frame.x * scale;
            projected.frame.y = origin_y + projected.frame.y * scale;
            projected.frame.width *= scale;
            projected.frame.height *= scale;
            projected.clip_frame = Some(match projected.clip_frame.take() {
                Some(mut own) => {
                    own.x = origin_x + own.x * scale;
                    own.y = origin_y + own.y * scale;
                    own.width *= scale;
                    own.height *= scale;
                    super::super::paint_geometry::intersect(&own, clip).unwrap_or_default()
                }
                None => clip.clone(),
            });
            projected.font_size *= scale;
            projected.line_height *= scale;
            projected.border_width *= scale;
            projected.corner_radius *= scale;
            if let Some(shadow) = &mut projected.box_shadow {
                shadow.offset_x *= scale;
                shadow.offset_y *= scale;
                shadow.blur_radius *= scale;
                shadow.spread_radius *= scale;
                shadow.radius *= scale;
            }
            projected.source_surface_frame = Some(std::sync::Arc::clone(source));
            projected.source_render_command_ref = command_ref;
        }
    }
    draw_host_paint_commands(frame, &host_commands)
}
