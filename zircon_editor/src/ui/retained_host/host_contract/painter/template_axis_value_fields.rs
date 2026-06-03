use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::theme::PALETTE;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const AXIS_FIELD_FONT_SIZE: f32 = 11.0;
const AXIS_FIELD_TEXT_INSET_X: f32 = 7.0;
const AXIS_FIELD_MAX_HEIGHT: f32 = 26.0;
const AXIS_FIELD_RADIUS: f32 = 4.0;
const AXIS_FIELD_BACKGROUND: [u8; 4] = [17, 22, 26, 255];
const AXIS_FIELD_HOVER_BACKGROUND: [u8; 4] = [23, 30, 35, 255];
const AXIS_FIELD_PRESSED_BACKGROUND: [u8; 4] = [18, 39, 47, 255];
const AXIS_FIELD_DISABLED_BACKGROUND: [u8; 4] = [21, 25, 29, 255];
const AXIS_FIELD_BORDER: [u8; 4] = [38, 48, 55, 255];
const AXIS_FIELD_HOVER_BORDER: [u8; 4] = [56, 70, 79, 255];
const AXIS_FIELD_DISABLED_BORDER: [u8; 4] = [42, 49, 55, 255];

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

fn axis_field_background(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        AXIS_FIELD_DISABLED_BACKGROUND
    } else if node.pressed {
        AXIS_FIELD_PRESSED_BACKGROUND
    } else if node.hovered || node.focused || node.selected {
        AXIS_FIELD_HOVER_BACKGROUND
    } else {
        AXIS_FIELD_BACKGROUND
    }
}

fn axis_field_border(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        AXIS_FIELD_DISABLED_BORDER
    } else if matches!(node.validation_level.as_str(), "error" | "danger") {
        PALETTE.error
    } else if matches!(node.validation_level.as_str(), "warning") {
        PALETTE.warning
    } else if node.focused || node.selected || node.pressed {
        PALETTE.focus_ring
    } else if node.hovered {
        AXIS_FIELD_HOVER_BORDER
    } else {
        AXIS_FIELD_BORDER
    }
}

fn axis_field_border_width(node: &TemplatePaneNodeData) -> f32 {
    if node.focused
        || node.selected
        || node.pressed
        || matches!(
            node.validation_level.as_str(),
            "error" | "danger" | "warning"
        )
    {
        1.5
    } else {
        1.0
    }
}

fn axis_field_text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        PALETTE.text_disabled
    } else if matches!(node.validation_level.as_str(), "error" | "danger") {
        PALETTE.error
    } else if node.value_color.a > 0 {
        [
            node.value_color.r,
            node.value_color.g,
            node.value_color.b,
            node.value_color.a,
        ]
    } else {
        PALETTE.text
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::data::TemplateNodeFrameData;
    use super::super::template_nodes::paint_template_nodes_for_test;
    use super::*;
    use crate::ui::layouts::common::model_rc;

    #[test]
    fn axis_value_field_kind_matches_transform_axis_inputs_only() {
        assert!(is_workbench_axis_value_field(&axis_node(
            "WorkbenchTransformPositionX",
            "128.4"
        )));
        assert!(is_workbench_axis_value_field(&axis_node(
            "WorkbenchTransformRotationZ",
            "0 deg"
        )));
        assert!(is_workbench_axis_value_field(&axis_node(
            "WorkbenchTransformScaleY",
            "1.00"
        )));
        assert!(!is_workbench_axis_value_field(&label_node(
            "WorkbenchTransformPositionAxisX",
            "X"
        )));
        assert!(!is_workbench_axis_value_field(&axis_node(
            "WorkbenchInputText",
            "Text field"
        )));
    }

    #[test]
    fn axis_value_field_paints_compact_field_and_value() {
        let bytes = paint_template_nodes_for_test(
            96,
            48,
            model_rc(vec![axis_node("WorkbenchTransformPositionX", "128.4")]),
        );

        assert_eq!(pixel_at(&bytes, 96, 22, 8), AXIS_FIELD_BORDER);
        assert_eq!(pixel_at(&bytes, 96, 60, 18), AXIS_FIELD_BACKGROUND);
        assert!(changed_pixel_count(&bytes, 96, 16, 12, 44, 18) > 0);
    }

    #[test]
    fn focused_axis_value_field_uses_focus_border() {
        let mut node = axis_node("WorkbenchTransformRotationY", "90 deg");
        node.focused = true;

        let bytes = paint_template_nodes_for_test(96, 48, model_rc(vec![node]));

        assert_eq!(pixel_at(&bytes, 96, 22, 8), PALETTE.focus_ring);
        assert_eq!(pixel_at(&bytes, 96, 18, 18), AXIS_FIELD_HOVER_BACKGROUND);
    }

    #[test]
    fn disabled_axis_value_field_uses_muted_surface() {
        let mut node = axis_node("WorkbenchTransformScaleZ", "1.00");
        node.disabled = true;

        let bytes = paint_template_nodes_for_test(96, 48, model_rc(vec![node]));

        assert_eq!(pixel_at(&bytes, 96, 22, 8), AXIS_FIELD_DISABLED_BORDER);
        assert_eq!(pixel_at(&bytes, 96, 60, 18), AXIS_FIELD_DISABLED_BACKGROUND);
    }

    #[test]
    fn axis_value_field_uses_declared_value_color_when_present() {
        let mut node = axis_node("WorkbenchTransformPositionX", "128.4");
        node.value_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(146, 158, 164);

        assert_eq!(axis_field_text_color(&node), [146, 158, 164, 255]);
    }

    fn axis_node(control_id: &str, value: &str) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: control_id.into(),
            role: "InputField".into(),
            component_role: "input-field".into(),
            value_text: value.into(),
            frame: TemplateNodeFrameData {
                x: 8.0,
                y: 8.0,
                width: 58.0,
                height: 24.0,
            },
            ..TemplatePaneNodeData::default()
        }
    }

    fn label_node(control_id: &str, text: &str) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: control_id.into(),
            role: "Label".into(),
            text: text.into(),
            ..TemplatePaneNodeData::default()
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
}
