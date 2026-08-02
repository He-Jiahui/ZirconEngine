use super::super::super::data::{FrameRect, TemplatePaneOptionData};
use super::super::super::paint_geometry::intersect;
use super::super::render_commands::HostPaintCommand;
#[cfg(test)]
use super::instrumentation::{record_message_text_copy, record_title_text_copy};
use super::layout::{
    NotificationCenterMetrics, mark_rect, message_rect, row_text_width, title_rect,
};
use super::style::{
    NotificationCenterPalette, row_background, row_border, severity_color, title_color,
};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_notification_row(
    commands: &mut Vec<HostPaintCommand>,
    option: &TemplatePaneOptionData,
    row_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    palette: NotificationCenterPalette,
    metrics: &NotificationCenterMetrics,
) {
    if row_rect.width <= 0.0 || row_rect.height <= 0.0 || intersect(row_rect, clip).is_none() {
        return;
    }

    commands.push(HostPaintCommand::quad(
        row_rect.clone(),
        Some(clip.clone()),
        order,
        Some(row_background(option, palette)),
        Some(row_border(option, palette)),
        metrics.border_width,
        metrics.row_radius,
        opacity,
    ));
    let mark = mark_rect(row_rect, metrics);
    if mark.width > 0.0 && mark.height > 0.0 {
        commands.push(HostPaintCommand::quad(
            mark,
            Some(clip.clone()),
            order + 1,
            Some(severity_color(option.tone.as_str(), palette)),
            None,
            0.0,
            metrics.mark_radius,
            opacity,
        ));
    }

    let text_width = row_text_width(row_rect, metrics);
    let title = title_rect(row_rect, text_width, metrics);
    if title.width > 0.0 && title.height > 0.0 {
        #[cfg(test)]
        record_title_text_copy();
        commands.push(HostPaintCommand::text(
            title,
            Some(clip.clone()),
            order + 2,
            option.label.to_string(),
            title_color(option, palette),
            metrics.title_font_size,
            metrics.title_line_height,
            UiTextRunPaintStyle::default(),
            opacity,
        ));
    }

    let description = option.description.as_str();
    if description.is_empty() {
        return;
    }
    let message = message_rect(row_rect, text_width, metrics);
    if message.width > 0.0 && message.height > 0.0 {
        #[cfg(test)]
        record_message_text_copy();
        commands.push(HostPaintCommand::text(
            message,
            Some(clip.clone()),
            order + 3,
            description.to_string(),
            palette.muted_text,
            metrics.message_font_size,
            metrics.message_line_height,
            UiTextRunPaintStyle::default(),
            opacity,
        ));
    }
}
