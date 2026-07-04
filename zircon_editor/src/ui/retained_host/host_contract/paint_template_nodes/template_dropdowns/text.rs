use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::WorkbenchDropdownStyle;
use super::super::template_dropdown_glyphs::dropdown_chevron_reserve;
use super::super::template_dropdown_metrics::workbench_dropdown_metrics;
use super::super::template_node_labels::template_node_label;
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
    let metrics = workbench_dropdown_metrics();
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: rect.x + metrics.text_inset_x,
            y: rect.y + (rect.height - metrics.line_height).max(0.0) * 0.5,
            width: (rect.width - metrics.text_inset_x - dropdown_chevron_reserve()).max(1.0),
            height: metrics.line_height,
        },
        Some(clip.clone()),
        order,
        label,
        style.text,
        metrics.font_size,
        metrics.line_height,
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
