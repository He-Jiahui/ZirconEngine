use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::WorkbenchDropdownStyle;
use super::super::template_dropdown_glyphs::DROPDOWN_CHEVRON_RESERVE;
use super::super::template_node_labels::template_node_label;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const DROPDOWN_FONT_SIZE: f32 = 11.0;
const DROPDOWN_LINE_HEIGHT: f32 = DROPDOWN_FONT_SIZE * 1.25;
const DROPDOWN_TEXT_LEFT: f32 = 10.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_dropdown_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    style: &WorkbenchDropdownStyle,
) {
    let label = dropdown_label(node);
    if label.trim().is_empty() {
        return;
    }
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: rect.x + DROPDOWN_TEXT_LEFT,
            y: rect.y + (rect.height - DROPDOWN_LINE_HEIGHT).max(0.0) * 0.5,
            width: (rect.width - DROPDOWN_TEXT_LEFT - DROPDOWN_CHEVRON_RESERVE).max(1.0),
            height: DROPDOWN_LINE_HEIGHT,
        },
        Some(clip.clone()),
        order,
        label,
        style.text,
        DROPDOWN_FONT_SIZE,
        DROPDOWN_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn dropdown_label(node: &TemplatePaneNodeData) -> String {
    let label = template_node_label(node, None);
    if !label.trim().is_empty() {
        return label;
    }
    node.options
        .row_data(0)
        .map(|value| value.to_string())
        .unwrap_or_default()
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dropdown_label_is_placeholder(
    node: &TemplatePaneNodeData,
) -> bool {
    template_node_label(node, None).trim().is_empty()
}
