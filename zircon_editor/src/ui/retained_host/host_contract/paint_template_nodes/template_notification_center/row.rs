use super::super::super::data::{FrameRect, TemplatePaneOptionData};
use super::super::super::paint_geometry::intersect;
use super::super::render_commands::HostPaintCommand;
use super::layout::{
    mark_rect, message_rect, row_text_width, title_rect, MARK_RADIUS, MESSAGE_FONT_SIZE,
    MESSAGE_LINE_HEIGHT, ROW_RADIUS, TITLE_FONT_SIZE, TITLE_LINE_HEIGHT,
};
use super::style::{row_background, row_border, severity_color, title_color, MUTED_TEXT};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_notification_row(
    commands: &mut Vec<HostPaintCommand>,
    option: &TemplatePaneOptionData,
    row_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if intersect(row_rect, clip).is_none() {
        return;
    }

    commands.push(HostPaintCommand::quad(
        row_rect.clone(),
        Some(clip.clone()),
        order,
        Some(row_background(option)),
        Some(row_border(option)),
        1.0,
        ROW_RADIUS,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        mark_rect(row_rect),
        Some(clip.clone()),
        order + 1,
        Some(severity_color(option.tone.as_str())),
        None,
        0.0,
        MARK_RADIUS,
        opacity,
    ));

    let text_width = row_text_width(row_rect);
    commands.push(HostPaintCommand::text(
        title_rect(row_rect, text_width),
        Some(clip.clone()),
        order + 2,
        option.label.to_string(),
        title_color(option),
        TITLE_FONT_SIZE,
        TITLE_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));

    let message = option.description.to_string();
    if message.is_empty() {
        return;
    }
    commands.push(HostPaintCommand::text(
        message_rect(row_rect, text_width),
        Some(clip.clone()),
        order + 3,
        message,
        MUTED_TEXT,
        MESSAGE_FONT_SIZE,
        MESSAGE_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
