use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::identity::{empty_text, header_text};
use super::layout::{empty_text_rect, header_rect, NotificationCenterMetrics};
use super::style::NotificationCenterPalette;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_notification_panel_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    palette: NotificationCenterPalette,
    metrics: &NotificationCenterMetrics,
) {
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(palette.panel_surface),
        Some(palette.panel_border),
        metrics.border_width,
        metrics.panel_radius,
        opacity,
    ));

    let header = header_rect(rect, metrics);
    if header.width > 0.0 && header.height > 0.0 {
        commands.push(HostPaintCommand::text(
            header,
            Some(clip.clone()),
            order + 1,
            header_text(node),
            palette.header_text,
            metrics.header_font_size,
            metrics.header_line_height,
            UiTextRunPaintStyle::default(),
            opacity,
        ));
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_empty_notification_message(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    palette: NotificationCenterPalette,
    metrics: &NotificationCenterMetrics,
) {
    let message = empty_text_rect(rect, metrics);
    if message.width > 0.0 && message.height > 0.0 {
        commands.push(HostPaintCommand::text(
            message,
            Some(clip.clone()),
            order,
            empty_text(node),
            palette.muted_text,
            metrics.message_font_size,
            metrics.message_line_height,
            UiTextRunPaintStyle::default(),
            opacity,
        ));
    }
}
