use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_theme::PALETTE;
use super::super::render_commands::HostPaintCommand;
use super::super::template_inspector_row_geometry::{
    nested_label_rect, INSPECTOR_LABEL_WIDTH, INSPECTOR_ROW_TEXT_Y,
};
use super::style::{
    resource_field_background, resource_field_border, INSPECTOR_FIELD_RADIUS, INSPECTOR_FONT_SIZE,
    INSPECTOR_LABEL_COLOR,
};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(super) fn push_label(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    label: &str,
    color: [u8; 4],
    opacity: f32,
) {
    push_text(
        commands,
        FrameRect {
            x: rect.x + 1.0,
            y: rect.y + INSPECTOR_ROW_TEXT_Y,
            width: INSPECTOR_LABEL_WIDTH - 4.0,
            height: (rect.height - INSPECTOR_ROW_TEXT_Y * 2.0).max(1.0),
        },
        clip,
        order,
        label,
        color,
        opacity,
    );
}

pub(super) fn push_nested_label(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    label: &str,
    opacity: f32,
) {
    push_text(
        commands,
        nested_label_rect(rect),
        clip,
        order,
        label,
        INSPECTOR_LABEL_COLOR,
        opacity,
    );
}

pub(super) fn push_field(
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
        Some(resource_field_background(node)),
        Some(if node.focused {
            PALETTE.focus_ring
        } else {
            resource_field_border(node)
        }),
        1.0,
        INSPECTOR_FIELD_RADIUS,
        opacity,
    ));
}

pub(super) fn push_text(
    commands: &mut Vec<HostPaintCommand>,
    rect: FrameRect,
    clip: &FrameRect,
    order: i32,
    text: &str,
    color: [u8; 4],
    opacity: f32,
) {
    if text.trim().is_empty() {
        return;
    }
    commands.push(HostPaintCommand::text(
        rect,
        Some(clip.clone()),
        order,
        text.to_string(),
        color,
        INSPECTOR_FONT_SIZE,
        INSPECTOR_FONT_SIZE * 1.2,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
