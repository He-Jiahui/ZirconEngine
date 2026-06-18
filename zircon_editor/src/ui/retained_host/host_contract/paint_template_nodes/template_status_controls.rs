use super::super::data::{FrameRect, TemplatePaneNodeData};
#[cfg(test)]
use super::super::paint_theme::PALETTE;
use super::render_commands::HostPaintCommand;
use super::style_selector::{
    select_workbench_status_chip_style, select_workbench_status_icon_button_style,
    select_workbench_status_signal_style, WorkbenchStatusSignalKind as StatusSignalKind,
};
#[cfg(test)]
use super::style_selector::{
    WorkbenchStatusSignalStyle, WORKBENCH_STATUS_NO_ERRORS_FILL as STATUS_NO_ERRORS_FILL,
    WORKBENCH_STATUS_RIGHT_BORDER as STATUS_RIGHT_BORDER,
};
use super::template_node_labels::template_node_label;
#[cfg(test)]
use super::template_status_control_geometry::status_signal_text_gap;
use super::template_status_control_geometry::{
    status_chip_chevron_rect, status_chip_text_rect, status_control_offset_rect,
    status_icon_button_glyph_rect, status_line_height, status_signal_icon_paint_rect,
    status_signal_icon_rect, status_signal_text_rect, STATUS_CHIP_RADIUS, STATUS_FONT_SIZE,
    STATUS_ICON_BUTTON_RADIUS,
};
#[cfg(test)]
use super::template_status_glyphs::warning_mark_segments;
use super::template_status_glyphs::{
    normalized_status_mark_width, push_down_chevron, push_status_icon_glyph,
    push_status_signal_icon, StatusIconKind,
};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(super) fn push_status_control_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    match status_control_kind(node) {
        Some(StatusControlKind::Signal(kind)) => {
            push_status_signal_item(commands, node, rect, clip, order, kind, opacity);
            true
        }
        Some(StatusControlKind::Chip) => {
            push_status_chip(commands, node, rect, clip, order, opacity);
            true
        }
        Some(StatusControlKind::Icon(kind)) => {
            push_status_icon_button(commands, node, rect, clip, order, kind, opacity);
            true
        }
        None => false,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StatusControlKind {
    Signal(StatusSignalKind),
    Chip,
    Icon(StatusIconKind),
}

fn status_control_kind(node: &TemplatePaneNodeData) -> Option<StatusControlKind> {
    match node.control_id.as_str() {
        "WorkbenchStatusReady" => Some(StatusControlKind::Signal(StatusSignalKind::Ready)),
        "WorkbenchStatusErrors" => Some(StatusControlKind::Signal(StatusSignalKind::Success)),
        "WorkbenchStatusWarnings" => Some(StatusControlKind::Signal(StatusSignalKind::Warning)),
        "WorkbenchStatusMessages" => Some(StatusControlKind::Signal(StatusSignalKind::Info)),
        "WorkbenchStatusGrid" | "WorkbenchStatusSnap" | "WorkbenchStatusZoom" => {
            Some(StatusControlKind::Chip)
        }
        "WorkbenchStatusSnapToggle" => Some(StatusControlKind::Icon(StatusIconKind::Snap)),
        "WorkbenchStatusWorld" => Some(StatusControlKind::Icon(StatusIconKind::World)),
        "WorkbenchStatusTarget" => Some(StatusControlKind::Icon(StatusIconKind::Target)),
        _ => None,
    }
}

fn push_status_signal_item(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    kind: StatusSignalKind,
    opacity: f32,
) {
    let style = select_workbench_status_signal_style(node, kind);
    let mark_width = status_signal_mark_width(node);
    let icon = status_signal_icon_rect(node, rect, kind);
    let icon_paint = status_signal_icon_paint_rect(node, &icon, kind);
    push_status_signal_icon(
        commands,
        &icon_paint,
        clip,
        order,
        kind,
        style,
        mark_width,
        opacity,
    );
    let label = template_node_label(node, None);
    if label.trim().is_empty() {
        return;
    }
    commands.push(HostPaintCommand::text(
        status_signal_text_rect(node, rect, &icon),
        Some(clip.clone()),
        order + 2,
        label,
        style.text,
        STATUS_FONT_SIZE,
        status_line_height(),
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn push_status_chip(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let rect = status_control_offset_rect(node, rect);
    let style = select_workbench_status_chip_style(node);
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(style.background),
        Some(style.border),
        1.0,
        STATUS_CHIP_RADIUS,
        opacity,
    ));

    let label = template_node_label(node, None);
    if !label.trim().is_empty() {
        commands.push(HostPaintCommand::text(
            status_chip_text_rect(&rect),
            Some(clip.clone()),
            order + 2,
            label,
            style.text,
            STATUS_FONT_SIZE,
            status_line_height(),
            UiTextRunPaintStyle::default(),
            opacity,
        ));
    }

    let chevron = status_chip_chevron_rect(&rect);
    push_down_chevron(commands, &chevron, clip, order + 3, style.text, opacity);
}

fn push_status_icon_button(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    kind: StatusIconKind,
    opacity: f32,
) {
    let rect = status_control_offset_rect(node, rect);
    let style = select_workbench_status_icon_button_style(node);
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(style.background),
        Some(style.border),
        1.0,
        STATUS_ICON_BUTTON_RADIUS,
        opacity,
    ));
    let glyph = status_icon_button_glyph_rect(&rect);
    push_status_icon_glyph(
        commands,
        &glyph,
        clip,
        order + 2,
        kind,
        style.glyph,
        opacity,
    );
}

#[cfg(test)]
fn status_signal_style(
    node: &TemplatePaneNodeData,
    kind: StatusSignalKind,
) -> WorkbenchStatusSignalStyle {
    select_workbench_status_signal_style(node, kind)
}

#[cfg(test)]
fn status_signal_icon_fill(node: &TemplatePaneNodeData, kind: StatusSignalKind) -> [u8; 4] {
    status_signal_style(node, kind).icon_fill
}

#[cfg(test)]
fn status_signal_text_color(node: &TemplatePaneNodeData, kind: StatusSignalKind) -> [u8; 4] {
    status_signal_style(node, kind).text
}

#[cfg(test)]
fn status_signal_mark_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    status_signal_style(node, StatusSignalKind::Ready).mark
}

fn status_signal_mark_width(node: &TemplatePaneNodeData) -> f32 {
    normalized_status_mark_width(node.icon_stroke_width)
}

#[cfg(test)]
fn status_chip_text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    select_workbench_status_chip_style(node).text
}

#[cfg(test)]
#[path = "template_status_controls_tests.rs"]
mod tests;
