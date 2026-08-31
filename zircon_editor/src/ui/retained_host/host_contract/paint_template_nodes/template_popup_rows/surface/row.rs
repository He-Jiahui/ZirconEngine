use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_geometry::intersect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::style_selector::WorkbenchPopupRowStyle;
use super::super::layers::popup_row_surface_order;
use super::super::metrics::workbench_popup_row_metrics;

mod style;

use style::popup_row_surface_command_style;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_popup_row_surface(
    commands: &mut Vec<HostPaintCommand>,
    row_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    style: WorkbenchPopupRowStyle,
    opacity: f32,
) {
    if intersect(row_rect, clip).is_none() {
        return;
    }
    let metrics = workbench_popup_row_metrics();
    let Some(style) = popup_row_surface_command_style(style, &metrics) else {
        return;
    };
    commands.push(HostPaintCommand::quad(
        row_rect.clone(),
        Some(clip.clone()),
        popup_row_surface_order(order),
        style.fill,
        style.border,
        style.border_width,
        style.radius,
        opacity,
    ));
}
