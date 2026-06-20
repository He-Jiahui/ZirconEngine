use super::super::super::render_commands::HostPaintCommand;
use super::super::super::style_selector::{
    select_workbench_status_signal_style, WorkbenchStatusSignalKind as StatusSignalKind,
};
use super::super::super::template_node_labels::template_node_label;
use super::super::super::template_status_control_geometry::{
    status_line_height, status_signal_icon_paint_rect, status_signal_icon_rect,
    status_signal_text_rect, STATUS_FONT_SIZE,
};
use super::super::super::template_status_glyphs::push_status_signal_icon;
use super::metrics::status_signal_mark_width;
use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_status_signal_item(
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
