use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::WorkbenchTextFieldStyle;
use super::metrics::workbench_field_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_field_surface(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    style: &WorkbenchTextFieldStyle,
) {
    let metrics = workbench_field_metrics();
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
