use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::paint_geometry::intersect;
use super::render_commands::HostPaintCommand;

mod identity;
mod layout;
mod style;
mod surface;
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_drag_overlay_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !identity::is_drag_overlay(node) {
        return false;
    }
    if !node.popup_open && !node.dragging {
        return true;
    }

    let metrics = layout::drag_overlay_metrics();
    let palette = style::drag_overlay_palette();
    let preview_rect = layout::preview_frame(node, rect);
    if preview_rect.width <= 0.0
        || preview_rect.height <= 0.0
        || intersect(&preview_rect, clip).is_none()
    {
        return true;
    }

    surface::push_preview_surface(
        commands,
        node,
        &preview_rect,
        clip,
        order,
        opacity,
        palette,
        &metrics,
    );
    surface::push_preview_icon(
        commands,
        node,
        &preview_rect,
        clip,
        order + 1,
        opacity,
        palette,
        &metrics,
    );
    text::push_preview_label(
        commands,
        node,
        &preview_rect,
        clip,
        order + 2,
        opacity,
        palette,
        &metrics,
    );

    if let Some(indicator) = layout::indicator_frame(node, &metrics) {
        surface::push_drop_indicator(commands, node, indicator, clip, order + 3, opacity, palette);
    }

    true
}
