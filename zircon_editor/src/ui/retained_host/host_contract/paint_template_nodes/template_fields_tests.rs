use super::super::super::data::TemplateNodeFrameData;
use super::super::style_selector::{
    WORKBENCH_TEXT_FIELD_BORDER as FIELD_BORDER,
    WORKBENCH_TEXT_FIELD_DISABLED_BORDER as FIELD_DISABLED_BORDER,
    WORKBENCH_TEXT_FIELD_DISABLED_SURFACE as FIELD_DISABLED_SURFACE,
    WORKBENCH_TEXT_FIELD_DISABLED_TEXT as FIELD_DISABLED_TEXT,
    WORKBENCH_TEXT_FIELD_FOCUSED_BORDER as FIELD_FOCUSED_BORDER,
    WORKBENCH_TEXT_FIELD_FOCUSED_SURFACE as FIELD_FOCUSED_SURFACE,
    WORKBENCH_TEXT_FIELD_PLACEHOLDER as FIELD_PLACEHOLDER,
    WORKBENCH_TEXT_FIELD_SURFACE as FIELD_SURFACE,
};
use super::super::template_field_stepper::STEPPER_DIVIDER;
use super::super::template_nodes::paint_template_nodes_for_test;
use super::*;
use crate::ui::layouts::common::model_rc;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

#[test]
fn workbench_field_matches_component_fields_but_not_axis_fields() {
    assert!(is_workbench_field(&field_node(
        "WorkbenchInputText",
        "Text field"
    )));
    assert!(is_workbench_field(&field_node("WorkbenchFieldRoot", "")));
    assert!(!is_workbench_field(&field_node(
        "WorkbenchTransformPositionX",
        "128.4"
    )));
}

#[test]
fn workbench_field_paints_surface_border_and_text() {
    let bytes = paint_template_nodes_for_test(
        200,
        48,
        model_rc(vec![positioned_field_node(
            "WorkbenchInputText",
            "Text field",
            12.0,
            8.0,
            170.0,
            32.0,
        )]),
    );

    assert_eq!(pixel_at(&bytes, 200, 170, 24), FIELD_SURFACE);
    assert_eq!(pixel_at(&bytes, 200, 80, 8), FIELD_BORDER);
    assert!(changed_pixel_count(&bytes, 200, 22, 16, 64, 18) > 0);
}

#[test]
fn focused_workbench_field_uses_focused_border() {
    let mut node = positioned_field_node(
        "WorkbenchInputFocused",
        "Focused input",
        12.0,
        8.0,
        170.0,
        32.0,
    );
    node.focused = true;
    let bytes = paint_template_nodes_for_test(200, 48, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 200, 80, 8), FIELD_FOCUSED_BORDER);
}

#[test]
fn focused_workbench_field_uses_declared_focus_border() {
    let mut node = positioned_field_node(
        "WorkbenchInputFocused",
        "Focused input",
        12.0,
        8.0,
        170.0,
        32.0,
    );
    node.focused = true;
    node.button_style.element.border_color =
        Some(zircon_runtime_interface::ui::style::UiStyleColor::Rgba(
            zircon_runtime_interface::ui::style::UiRgbaColor::from_u8(27, 152, 160, 255),
        ));

    assert_eq!(field_border(&node), [27, 152, 160, 255]);
}

#[test]
fn disabled_workbench_field_paints_placeholder_tone() {
    let mut node = positioned_field_node("WorkbenchInputDisabled", "", 12.0, 8.0, 170.0, 32.0);
    node.disabled = true;
    let text_color = field_text_color(&node);
    let bytes = paint_template_nodes_for_test(200, 48, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 200, 170, 24), FIELD_DISABLED_SURFACE);
    assert_eq!(pixel_at(&bytes, 200, 80, 8), FIELD_DISABLED_BORDER);
    assert_eq!(text_color, FIELD_DISABLED_TEXT);
    assert!(changed_pixel_count(&bytes, 200, 22, 16, 90, 18) > 0);
}

#[test]
fn disabled_workbench_field_uses_declared_opacity() {
    let mut node = positioned_field_node("WorkbenchInputDisabled", "", 12.0, 8.0, 170.0, 32.0);
    node.disabled = true;
    node.button_style.element.opacity = 0.94;

    assert!((field_opacity(&node, 1.0) - 0.94).abs() < 0.001);
    assert!((field_opacity(&node, 0.5) - 0.47).abs() < 0.001);
}

#[test]
fn workbench_field_selector_uses_shared_text_field_state_priority() {
    let mut node =
        positioned_field_node("WorkbenchInputText", "Text field", 12.0, 8.0, 170.0, 32.0);
    node.hovered = true;
    node.focused = true;
    node.pressed = true;

    assert_eq!(field_visual_state(&node), UiPainterResolvedState::Pressed);
    assert_eq!(field_surface(&node), FIELD_FOCUSED_SURFACE);

    node.pressed = false;
    assert_eq!(field_visual_state(&node), UiPainterResolvedState::Focused);
    assert_eq!(field_border(&node), FIELD_FOCUSED_BORDER);

    node.disabled = true;
    assert_eq!(field_visual_state(&node), UiPainterResolvedState::Disabled);
    assert_eq!(field_surface(&node), FIELD_DISABLED_SURFACE);
    assert_eq!(field_text_color(&node), FIELD_DISABLED_TEXT);

    let placeholder = positioned_field_node("WorkbenchInputDisabled", "", 12.0, 8.0, 170.0, 32.0);
    assert_eq!(field_text_color(&placeholder), FIELD_PLACEHOLDER);
}

#[test]
fn stepper_workbench_field_paints_right_arrows() {
    let bytes = paint_template_nodes_for_test(
        112,
        48,
        model_rc(vec![positioned_field_node(
            "WorkbenchInputStepper",
            "42",
            12.0,
            8.0,
            67.0,
            32.0,
        )]),
    );

    assert_eq!(pixel_at(&bytes, 112, 61, 16), STEPPER_DIVIDER);
    assert!(changed_pixel_count(&bytes, 112, 64, 15, 12, 20) > 0);
}

#[test]
fn stepper_workbench_field_honors_declared_layout_offset() {
    let mut node = positioned_field_node("WorkbenchInputStepper", "42", 12.0, 8.0, 67.0, 32.0);
    node.layout_offset_x = 5.0;
    node.layout_offset_y = 6.0;
    let bytes = paint_template_nodes_for_test(128, 72, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 128, 66, 20), STEPPER_DIVIDER);
    assert_eq!(pixel_at(&bytes, 128, 14, 24), [0, 0, 0, 255]);
}

#[test]
fn workbench_field_preserves_half_pixel_declared_height() {
    let rect = pixel_aligned_rect(&FrameRect {
        x: 12.3,
        y: 8.4,
        width: 67.2,
        height: 30.5,
    });

    assert_eq!(rect.x, 12.0);
    assert_eq!(rect.y, 8.0);
    assert_eq!(rect.width, 67.0);
    assert_eq!(rect.height, 30.5);
}

fn field_surface(node: &TemplatePaneNodeData) -> [u8; 4] {
    field_style(node).surface
}

fn field_border(node: &TemplatePaneNodeData) -> [u8; 4] {
    field_style(node).border
}

fn field_text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    field_style(node).text
}

fn field_visual_state(node: &TemplatePaneNodeData) -> UiPainterResolvedState {
    field_style(node).state
}

fn field_node(control_id: &str, value: &str) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "InputField".into(),
        component_role: "input-field".into(),
        value_text: value.into(),
        frame: TemplateNodeFrameData {
            x: 0.0,
            y: 0.0,
            width: 170.0,
            height: 32.0,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn positioned_field_node(
    control_id: &str,
    value: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..field_node(control_id, value)
    }
}

fn changed_pixel_count(
    bytes: &[u8],
    frame_width: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> usize {
    let mut changed = 0;
    for py in y..(y + height) {
        for px in x..(x + width) {
            let index = ((py as usize * frame_width as usize) + px as usize) * 4;
            if bytes[index..index + 4] != [0, 0, 0, 255] {
                changed += 1;
            }
        }
    }
    changed
}

fn pixel_at(bytes: &[u8], frame_width: u32, x: u32, y: u32) -> [u8; 4] {
    let index = ((y as usize * frame_width as usize) + x as usize) * 4;
    [
        bytes[index],
        bytes[index + 1],
        bytes[index + 2],
        bytes[index + 3],
    ]
}
