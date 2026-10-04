mod geometry;
mod style;

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_icon_assets::push_icon_asset_pixels;
use super::layers::scale_link_connector_order;
use super::metrics::axis_label_metrics;

use geometry::{scale_link_asset_frame, scale_link_origin_with_metrics};
use style::scale_link_asset_tint;

const SCALE_LINK_ICON: &str = "zircon_editor_shell/inspector/link.svg";

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_scale_link(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let metrics = axis_label_metrics();
    let frame = scale_link_asset_frame(node, rect, &metrics);
    let _ = push_icon_asset_pixels(
        commands,
        SCALE_LINK_ICON,
        &frame,
        clip,
        scale_link_connector_order(order),
        Some(scale_link_asset_tint(node)),
        opacity,
    );
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn scale_link_origin(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> (f32, f32) {
    scale_link_origin_with_metrics(node, rect, &axis_label_metrics())
}
