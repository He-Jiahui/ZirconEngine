//! 搜索底面的焦点反馈采用节点的可见焦点状态，避免把鼠标激活当作键盘焦点环。

use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::super::super::style_selector::focus_visible_for_node;
use super::super::super::layout::command_palette_metrics;
use super::super::super::palette::WorkbenchCommandPalettePalette;

mod style;

use style::command_palette_search_surface_style;

pub(super) fn push_command_palette_search_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    search_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    palette: &WorkbenchCommandPalettePalette,
) {
    let metrics = command_palette_metrics();
    let style =
        command_palette_search_surface_style(palette, &metrics, focus_visible_for_node(node));
    commands.push(HostPaintCommand::quad(
        search_rect.clone(),
        Some(clip.clone()),
        order,
        Some(style.fill),
        Some(style.border),
        style.border_width,
        style.radius,
        opacity,
    ));
}
