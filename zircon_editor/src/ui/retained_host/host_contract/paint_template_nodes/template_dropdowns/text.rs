use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::WorkbenchDropdownStyle;
use super::super::template_dropdown_glyphs::dropdown_chevron_reserve;
use super::super::template_node_labels::template_node_label;
use crate::ui::retained_host::host_contract::paint_theme::METRICS;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

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
    let line_height = METRICS.line_height(METRICS.font_body);
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: rect.x + METRICS.input_pad[0],
            y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
            width: (rect.width - METRICS.input_pad[0] - dropdown_chevron_reserve()).max(1.0),
            height: line_height,
        },
        Some(clip.clone()),
        order,
        label,
        style.text,
        METRICS.font_body,
        line_height,
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
