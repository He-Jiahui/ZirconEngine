use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_theme::current_host_metrics;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::style_selector::focus_visible_for_node;
use super::super::super::template_inspector_row_geometry::is_paintable_rect;
use super::super::style::{
    inspector_row_palette, resource_field_background_from_palette,
    resource_field_border_from_palette,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_field(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if !is_paintable_rect(rect) {
        return;
    }
    let palette = inspector_row_palette();
    let metrics = current_host_metrics();
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(resource_field_background_from_palette(node, palette)),
        Some(if focus_visible_for_node(node) {
            palette.focus_border
        } else {
            resource_field_border_from_palette(node, palette)
        }),
        metrics.border_width,
        metrics.radius_control,
        opacity,
    ));
}
