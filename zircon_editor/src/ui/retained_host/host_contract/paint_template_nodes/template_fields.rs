use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::template_component_family::{
    is_component_family, uses_workbench_visual_language, TemplateComponentFamily,
};
use super::render_commands::HostPaintCommand;
use super::style_selector::{select_workbench_text_field_style, WorkbenchTextFieldStyle};
#[cfg(test)]
#[path = "template_fields_tests.rs"]
mod tests;
use super::template_field_stepper::{push_field_stepper, STEPPER_WIDTH};
use super::template_node_labels::template_node_label;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const FIELD_FONT_SIZE: f32 = 11.0;
const FIELD_LINE_HEIGHT: f32 = FIELD_FONT_SIZE * 1.25;
const FIELD_RADIUS: f32 = 4.0;
const FIELD_TEXT_LEFT: f32 = 10.0;
const FIELD_TEXT_RIGHT: f32 = 8.0;

pub(super) fn push_field_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_workbench_field(node) {
        return false;
    }
    let rect = field_paint_rect(node, rect);
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return true;
    }
    let opacity = field_opacity(node, opacity);
    let style = field_style(node);

    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(style.surface),
        Some(style.border),
        1.0,
        FIELD_RADIUS,
        opacity,
    ));

    let stepper = is_stepper_field(node);
    if stepper {
        push_field_stepper(commands, &rect, clip, order + 2, opacity, &style);
    }
    push_field_text(
        commands,
        node,
        &rect,
        clip,
        order + 3,
        stepper,
        opacity,
        &style,
    );
    true
}

fn field_paint_rect(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    let mut rect = pixel_aligned_rect(rect);
    rect.x += node.layout_offset_x;
    rect.y += node.layout_offset_y;
    rect
}

fn is_workbench_field(node: &TemplatePaneNodeData) -> bool {
    uses_workbench_visual_language(node)
        && !node.control_id.as_str().starts_with("WorkbenchTransform")
        && is_component_family(node, TemplateComponentFamily::TextInput)
}

fn push_field_text(
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

fn field_opacity(node: &TemplatePaneNodeData, inherited_opacity: f32) -> f32 {
    (inherited_opacity * node.button_style.element.opacity).clamp(0.0, 1.0)
}

fn field_style(node: &TemplatePaneNodeData) -> WorkbenchTextFieldStyle {
    select_workbench_text_field_style(node, field_label_is_placeholder(node))
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

fn field_label_is_placeholder(node: &TemplatePaneNodeData) -> bool {
    template_node_label(node, None).trim().is_empty()
        && node.control_id.as_str() == "WorkbenchInputDisabled"
}

fn is_stepper_field(node: &TemplatePaneNodeData) -> bool {
    node.control_id.as_str() == "WorkbenchInputStepper"
}

fn pixel_aligned_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: rect.width.round().max(1.0),
        height: rect.height.max(1.0),
    }
}
