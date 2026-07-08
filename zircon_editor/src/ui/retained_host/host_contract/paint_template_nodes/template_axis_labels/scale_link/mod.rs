mod geometry;
mod style;

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::layers::scale_link_connector_order;
use super::metrics::axis_label_metrics;

use geometry::{scale_link_geometry, scale_link_origin_with_metrics};
use style::scale_link_command_style;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_scale_link(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let metrics = axis_label_metrics();
    let geometry = scale_link_geometry(node, rect, &metrics);
    let style = scale_link_command_style(node, &metrics);
    for lobe in geometry.lobes {
        commands.push(HostPaintCommand::quad(
            lobe,
            Some(clip.clone()),
            order,
            style.lobe.fill,
            style.lobe.border,
            style.lobe.border_width,
            style.lobe.radius,
            opacity,
        ));
    }
    commands.push(HostPaintCommand::quad(
        geometry.connector,
        Some(clip.clone()),
        scale_link_connector_order(order),
        style.connector.fill,
        style.connector.border,
        style.connector.border_width,
        style.connector.radius,
        opacity,
    ));
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn scale_link_origin(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> (f32, f32) {
    scale_link_origin_with_metrics(node, rect, &axis_label_metrics())
}
