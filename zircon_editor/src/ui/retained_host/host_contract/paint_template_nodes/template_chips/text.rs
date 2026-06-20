use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_chip_glyphs::{chip_has_chevron, CHIP_CHEVRON_RESERVE};
use super::super::template_node_labels::template_node_label;
use super::geometry::{chip_label_rect, CHIP_TEXT_RIGHT};
use super::style::{chip_text_color, CHIP_FONT_SIZE, CHIP_LINE_HEIGHT};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_chip_label(
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
    let right_reserve = if chip_has_chevron(node) {
        CHIP_CHEVRON_RESERVE
    } else {
        CHIP_TEXT_RIGHT
    };
    commands.push(HostPaintCommand::text(
        chip_label_rect(rect, right_reserve),
        Some(clip.clone()),
        order,
        label,
        chip_text_color(node),
        CHIP_FONT_SIZE,
        CHIP_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
