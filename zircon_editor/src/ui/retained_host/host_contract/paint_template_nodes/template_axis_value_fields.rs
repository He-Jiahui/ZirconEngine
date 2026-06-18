use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::template_axis_value_field_style::{
    axis_field_background, axis_field_border, axis_field_border_width, axis_field_text_color,
};
#[cfg(test)]
#[path = "template_axis_value_fields_tests.rs"]
mod tests;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const AXIS_FIELD_FONT_SIZE: f32 = 11.0;
const AXIS_FIELD_TEXT_INSET_X: f32 = 7.0;
const AXIS_FIELD_MAX_HEIGHT: f32 = 26.0;
const AXIS_FIELD_RADIUS: f32 = 4.0;

pub(super) fn push_axis_value_field_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_workbench_axis_value_field(node) {
        return false;
    }

    let field = axis_field_rect(rect);
    if field.width <= 0.0 || field.height <= 0.0 {
        return true;
    }

    commands.push(HostPaintCommand::quad(
        field.clone(),
        Some(clip.clone()),
        order,
        Some(axis_field_background(node)),
        Some(axis_field_border(node)),
        axis_field_border_width(node),
        AXIS_FIELD_RADIUS,
        opacity,
    ));

    let value = axis_field_value(node);
    if value.is_empty() {
        return true;
    }

    let line_height = AXIS_FIELD_FONT_SIZE * 1.2;
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: field.x + AXIS_FIELD_TEXT_INSET_X,
            y: field.y + (field.height - line_height).max(0.0) * 0.5,
            width: (field.width - AXIS_FIELD_TEXT_INSET_X * 2.0).max(1.0),
            height: line_height,
        },
        Some(clip.clone()),
        order + 1,
        value.to_string(),
        axis_field_text_color(node),
        AXIS_FIELD_FONT_SIZE,
        line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
    true
}

fn is_workbench_axis_value_field(node: &TemplatePaneNodeData) -> bool {
    if !is_text_input_node(node) {
        return false;
    }
    let control_id = node.control_id.as_str();
    control_id == "WorkbenchAxisValueFieldRoot"
        || transform_axis_value_id(control_id).is_some()
        || node.component_role.as_str() == "axis-value-field"
}

fn transform_axis_value_id(control_id: &str) -> Option<TransformAxisValueId> {
    let field = control_id.strip_prefix("WorkbenchTransform")?;
    let axis = if field.ends_with('X') {
        "X"
    } else if field.ends_with('Y') {
        "Y"
    } else if field.ends_with('Z') {
        "Z"
    } else {
        return None;
    };
    if field
        .strip_suffix(axis)
        .is_some_and(|kind| matches!(kind, "Position" | "Rotation" | "Scale"))
    {
        Some(TransformAxisValueId)
    } else {
        None
    }
}

#[derive(Clone, Copy)]
struct TransformAxisValueId;

fn is_text_input_node(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.role.as_str(),
        "InputField" | "LineEdit" | "TextField" | "MuiTextField"
    ) || matches!(
        node.component_role.as_str(),
        "input-field" | "number-field" | "text-field"
    )
}

fn axis_field_rect(rect: &FrameRect) -> FrameRect {
    let height = rect.height.min(AXIS_FIELD_MAX_HEIGHT).round().max(0.0);
    FrameRect {
        x: rect.x.round(),
        y: (rect.y + (rect.height - height).max(0.0) * 0.5).round(),
        width: rect.width.round().max(0.0),
        height,
    }
}

fn axis_field_value(node: &TemplatePaneNodeData) -> &str {
    let value = node.value_text.trim();
    if value.is_empty() {
        node.text.trim()
    } else {
        value
    }
}
