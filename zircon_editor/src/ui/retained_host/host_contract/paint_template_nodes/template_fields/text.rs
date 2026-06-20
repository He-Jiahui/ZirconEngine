use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::WorkbenchTextFieldStyle;
use super::super::template_field_stepper::STEPPER_WIDTH;
use super::super::template_node_labels::template_node_label;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const FIELD_FONT_SIZE: f32 = 11.0;
const FIELD_LINE_HEIGHT: f32 = FIELD_FONT_SIZE * 1.25;
const FIELD_TEXT_LEFT: f32 = 10.0;
const FIELD_TEXT_RIGHT: f32 = 8.0;

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
    let right_reserve = if stepper {
        STEPPER_WIDTH + FIELD_TEXT_RIGHT
    } else {
        FIELD_TEXT_RIGHT
    };
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: rect.x + FIELD_TEXT_LEFT,
            y: rect.y + (rect.height - FIELD_LINE_HEIGHT).max(0.0) * 0.5,
            width: (rect.width - FIELD_TEXT_LEFT - right_reserve).max(1.0),
            height: FIELD_LINE_HEIGHT,
        },
        Some(clip.clone()),
        order,
        label,
        style.text,
        FIELD_FONT_SIZE,
        FIELD_LINE_HEIGHT,
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
    template_node_label(node, None).trim().is_empty()
        && node.control_id.as_str() == "WorkbenchInputDisabled"
}
