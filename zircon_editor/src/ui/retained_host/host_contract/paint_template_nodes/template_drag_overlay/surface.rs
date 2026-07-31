use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::{layout, style};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_preview_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    preview_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    palette: style::DragOverlayPalette,
    metrics: &layout::DragOverlayMetrics,
) {
    commands.push(HostPaintCommand::quad(
        preview_rect.clone(),
        Some(clip.clone()),
        order,
        Some(style::preview_surface_color(node, palette)),
        Some(style::preview_accent_color(node, palette)),
        metrics.border_width,
        metrics.preview_radius,
        opacity,
    ));
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_preview_icon(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    preview_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    palette: style::DragOverlayPalette,
    metrics: &layout::DragOverlayMetrics,
) {
    let icon = layout::preview_icon_frame(preview_rect, metrics);
    if icon.width <= 0.0 || icon.height <= 0.0 {
        return;
    }
    commands.push(HostPaintCommand::quad(
        icon,
        Some(clip.clone()),
        order,
        Some(style::preview_accent_color(node, palette)),
        None,
        0.0,
        metrics.icon_radius,
        opacity,
    ));
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_drop_indicator(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    indicator: FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    palette: style::DragOverlayPalette,
) {
    commands.push(HostPaintCommand::quad(
        indicator,
        Some(clip.clone()),
        order,
        Some(style::preview_accent_color(node, palette)),
        None,
        0.0,
        1.0,
        opacity,
    ));
}
