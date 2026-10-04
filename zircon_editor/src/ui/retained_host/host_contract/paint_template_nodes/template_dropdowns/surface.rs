//! 使用入口的一次性样式和密度快照绘制下拉表面；调用方负责外框与 clip 的可见性。

use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::WorkbenchDropdownStyle;
use super::super::template_dropdown_metrics::WorkbenchDropdownMetrics;
use super::geometry::dropdown_surface_radius;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_dropdown_surface(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    style: &WorkbenchDropdownStyle,
    metrics: &WorkbenchDropdownMetrics,
) {
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(style.surface),
        Some(style.border),
        metrics.border_width,
        dropdown_surface_radius(rect, metrics),
        opacity,
    ));
}
