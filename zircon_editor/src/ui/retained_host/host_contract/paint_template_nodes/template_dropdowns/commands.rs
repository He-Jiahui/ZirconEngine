use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_dropdown_metrics::workbench_dropdown_metrics;
use super::geometry::{dropdown_chevron_fits, dropdown_paint_rect, has_paintable_dropdown_extent};
use super::identity::is_workbench_dropdown;
use super::layers::{chevron_order, label_order};
use super::style::dropdown_style;
use super::surface::push_dropdown_surface;
use super::text::{dropdown_label, push_dropdown_label};
use crate::ui::retained_host::host_contract::paint_geometry::intersect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_dropdown_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_workbench_dropdown(node) {
        return false;
    }
    let rect = dropdown_paint_rect(node, rect);
    if !has_paintable_dropdown_extent(&rect) || intersect(&rect, clip).is_none() {
        return true;
    }
    let (label, label_is_placeholder) = dropdown_label(node);
    let metrics = workbench_dropdown_metrics();
    let style = dropdown_style(node, label_is_placeholder);

    push_dropdown_surface(commands, &rect, clip, order, opacity, &style, &metrics);
    push_dropdown_label(
        commands,
        label,
        &rect,
        clip,
        label_order(order),
        opacity,
        &style,
        &metrics,
    );
    if dropdown_chevron_fits(&rect, &metrics) {
        super::super::template_dropdown_glyphs::push_dropdown_chevron(
            commands,
            &rect,
            clip,
            chevron_order(order),
            opacity,
            &style,
            &metrics,
        );
    }
    true
}
