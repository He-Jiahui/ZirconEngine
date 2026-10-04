//! 提交命令面板容器底面；上层已确认组件打开并提供 node clip，这里不扩大可绘制区域。

use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::layout::command_palette_metrics;
use super::super::palette::command_palette_palette;

mod style;

use style::command_palette_panel_surface_style;

pub(super) fn push_command_palette_panel_surface(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let metrics = command_palette_metrics();
    let palette = command_palette_palette();
    let style = command_palette_panel_surface_style(&palette, &metrics);
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(style.fill),
        Some(style.border),
        style.border_width,
        style.radius,
        opacity,
    ));
}
