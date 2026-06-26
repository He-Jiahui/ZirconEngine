use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_node_labels::template_node_label;
use super::super::template_row_metrics::{
    row_text_line_height, ROW_RIGHT_RESERVE, ROW_TEXT_FONT_SIZE, ROW_TEXT_INSET_X, ROW_TEXT_INSET_Y,
};
use super::style::list_row_text_color;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_list_row_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let label = template_node_label(node, None);
    if label.trim().is_empty() {
        return;
    }
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: rect.x + ROW_TEXT_INSET_X,
            y: rect.y + ROW_TEXT_INSET_Y,
            width: (rect.width - ROW_TEXT_INSET_X - ROW_RIGHT_RESERVE).max(1.0),
            height: (rect.height - ROW_TEXT_INSET_Y * 2.0).max(1.0),
        },
        Some(clip.clone()),
        order,
        label,
        list_row_text_color(node),
        ROW_TEXT_FONT_SIZE,
        row_text_line_height(),
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
