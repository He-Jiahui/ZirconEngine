use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_geometry::intersect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::layers::popup_separator_order;
use super::super::metrics::workbench_popup_row_metrics;

mod geometry;
mod style;

use geometry::popup_separator_rect;
use style::popup_separator_style;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_popup_separator(
    commands: &mut Vec<HostPaintCommand>,
    row_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let metrics = workbench_popup_row_metrics();
    let separator = popup_separator_rect(row_rect, &metrics);
    if intersect(&separator, clip).is_none() {
        return;
    }
    let style = popup_separator_style();
    commands.push(HostPaintCommand::quad(
        separator,
        Some(clip.clone()),
        popup_separator_order(order),
        Some(style.fill),
        style.border,
        style.border_width,
        style.radius,
        opacity,
    ));
}
