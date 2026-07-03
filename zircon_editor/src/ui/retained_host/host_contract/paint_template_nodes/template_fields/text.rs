use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::WorkbenchTextFieldStyle;
use super::super::template_field_stepper::STEPPER_WIDTH;
use super::super::template_node_labels::template_node_label;
use super::search::{search_field_label_is_placeholder, search_field_text_left};
use crate::ui::retained_host::host_contract::paint_theme::current_host_metrics;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_field_text(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    stepper: bool,
    opacity: f32,
    style: &WorkbenchTextFieldStyle,
) {
    let label = field_label(node);
    if label.trim().is_empty() {
        return;
    }
    let metrics = current_host_metrics();
    let line_height = metrics.line_height(metrics.font_body);
    let right_reserve = if stepper {
        STEPPER_WIDTH + metrics.input_pad[1]
    } else {
        metrics.input_pad[1]
    };
    let text_left = search_field_text_left(node);
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: rect.x + text_left,
            y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
            width: (rect.width - text_left - right_reserve).max(1.0),
            height: line_height,
        },
        Some(clip.clone()),
        order,
        label,
        style.text,
        metrics.font_body,
        line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn field_label(node: &TemplatePaneNodeData) -> String {
    let label = template_node_label(node, None);
    if !label.trim().is_empty() {
        return label;
    }
    match node.control_id.as_str() {
        "WorkbenchInputDisabled" => "Disabled input".to_string(),
        _ => String::new(),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn field_label_is_placeholder(
    node: &TemplatePaneNodeData,
) -> bool {
    let label = template_node_label(node, None);
    (label.trim().is_empty() && node.control_id.as_str() == "WorkbenchInputDisabled")
        || search_field_label_is_placeholder(node, &label)
        || import_path_field_label_is_placeholder(node, &label)
}

fn import_path_field_label_is_placeholder(node: &TemplatePaneNodeData, label: &str) -> bool {
    node.control_id.as_str() == "AssetBrowserImportPathField"
        && node.value_text.trim().is_empty()
        && !label.trim().is_empty()
}
