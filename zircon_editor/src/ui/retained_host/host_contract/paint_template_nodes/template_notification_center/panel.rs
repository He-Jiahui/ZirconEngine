use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::identity::{empty_text, header_text};
use super::layout::{
    empty_text_rect, header_rect, HEADER_FONT_SIZE, HEADER_LINE_HEIGHT, MESSAGE_FONT_SIZE,
    MESSAGE_LINE_HEIGHT, PANEL_RADIUS,
};
use super::style::{HEADER_TEXT, MUTED_TEXT, PANEL_BORDER, PANEL_SURFACE};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_notification_panel_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(PANEL_SURFACE),
        Some(PANEL_BORDER),
        1.0,
        PANEL_RADIUS,
        opacity,
    ));

    commands.push(HostPaintCommand::text(
        header_rect(rect),
        Some(clip.clone()),
        order + 1,
        header_text(node),
        HEADER_TEXT,
        HEADER_FONT_SIZE,
        HEADER_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_empty_notification_message(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::text(
        empty_text_rect(rect),
        Some(clip.clone()),
        order,
        empty_text(node),
        MUTED_TEXT,
        MESSAGE_FONT_SIZE,
        MESSAGE_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
