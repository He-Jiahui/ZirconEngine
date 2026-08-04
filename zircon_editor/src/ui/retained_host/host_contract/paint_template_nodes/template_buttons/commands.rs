use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::content::push_button_content;
use super::geometry::{button_paint_rect, has_paintable_button_extent};
use super::identity::{button_kind, is_workbench_button};
use super::layers::content_order;
use super::style::button_opacity;
use super::surface::push_button_surface;
use crate::ui::retained_host::host_contract::paint_geometry::intersect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_button_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_workbench_button(node) {
        return false;
    }
    if !has_paintable_button_extent(rect) {
        return true;
    }
    let rect = button_paint_rect(node, rect);
    if intersect(&rect, clip).is_none() {
        return true;
    }

    let kind = button_kind(node);
    let opacity = button_opacity(node, opacity);
    push_button_surface(commands, node, &rect, clip, order, kind, opacity);
    push_button_content(
        commands,
        node,
        &rect,
        clip,
        content_order(order),
        kind,
        opacity,
    );
    true
}
