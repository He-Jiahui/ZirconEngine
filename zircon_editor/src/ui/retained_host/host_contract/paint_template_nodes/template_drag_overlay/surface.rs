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
) {
    commands.push(HostPaintCommand::quad(
        preview_rect.clone(),
        Some(clip.clone()),
        order,
        Some(style::preview_surface_color(node)),
        Some(style::preview_accent_color(node)),
        1.0,
        layout::PREVIEW_RADIUS,
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
) {
    commands.push(HostPaintCommand::quad(
        layout::preview_icon_frame(preview_rect),
        Some(clip.clone()),
        order,
        Some(style::preview_accent_color(node)),
        None,
        0.0,
        layout::ICON_RADIUS,
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
) {
    commands.push(HostPaintCommand::quad(
        indicator,
        Some(clip.clone()),
        order,
        Some(style::preview_accent_color(node)),
        None,
        0.0,
        1.0,
        opacity,
    ));
}
