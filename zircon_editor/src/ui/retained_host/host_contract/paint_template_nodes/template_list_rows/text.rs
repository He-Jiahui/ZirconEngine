use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_node_labels::template_node_label;
use super::style::list_row_text_color;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const LIST_ROW_FONT_SIZE: f32 = 12.0;
const LIST_ROW_TEXT_INSET_X: f32 = 14.0;
const LIST_ROW_TEXT_INSET_Y: f32 = 6.0;
const LIST_ROW_ADORNMENT_RESERVE: f32 = 26.0;

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
            x: rect.x + LIST_ROW_TEXT_INSET_X,
            y: rect.y + LIST_ROW_TEXT_INSET_Y,
            width: (rect.width - LIST_ROW_TEXT_INSET_X - LIST_ROW_ADORNMENT_RESERVE).max(1.0),
            height: (rect.height - LIST_ROW_TEXT_INSET_Y * 2.0).max(1.0),
        },
        Some(clip.clone()),
        order,
        label,
        list_row_text_color(node),
        LIST_ROW_FONT_SIZE,
        LIST_ROW_FONT_SIZE * 1.2,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
