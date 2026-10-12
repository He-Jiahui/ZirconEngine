//! 集中 chrome 选择器负责 fill/分隔色，表面绘制器组合可选背景、内容框和更高层分隔线。

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_geometry::intersect;
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::select_workbench_chrome_style;
use super::frame::{shell_panel_border_color, shell_panel_border_width, shell_panel_corner_radius};
use super::identity::ShellPanelKind;
use super::separators::push_shell_panel_separators;

/// 由已认领 shell kind 的入口调用；clip 为上游已裁定的区域，缺少背景仍可尝试绘制分隔线。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_shell_panel_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    kind: ShellPanelKind,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if intersect(rect, clip).is_none() {
        return;
    }
    let style = select_workbench_chrome_style(node, kind);
    if let Some(fill) = style.fill {
        commands.push(HostPaintCommand::quad(
            rect.clone(),
            Some(clip.clone()),
            order,
            Some(fill),
            shell_panel_border_color(kind, &style),
            shell_panel_border_width(kind),
            shell_panel_corner_radius(kind),
            opacity,
        ));
    }
    push_shell_panel_separators(commands, kind, &style, rect, clip, order + 1, opacity);
}
