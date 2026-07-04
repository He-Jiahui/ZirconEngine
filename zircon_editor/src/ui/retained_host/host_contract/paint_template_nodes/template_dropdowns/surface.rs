use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::WorkbenchDropdownStyle;
use super::super::template_dropdown_metrics::workbench_dropdown_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_dropdown_surface(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    style: &WorkbenchDropdownStyle,
) {
    let metrics = workbench_dropdown_metrics();
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(style.surface),
        Some(style.border),
        metrics.border_width,
        metrics.radius,
        opacity,
    ));
}
