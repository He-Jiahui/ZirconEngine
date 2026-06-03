use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::template_style::text_color;
use super::theme::PALETTE;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const PROPERTY_LABEL_WIDTH: f32 = 83.0;
const PROPERTY_LABEL_MIN_WIDTH: f32 = 56.0;
const PROPERTY_LABEL_MAX_WIDTH_RATIO: f32 = 0.45;
const PROPERTY_TEXT_INSET_X: f32 = 5.0;
const PROPERTY_TEXT_INSET_Y: f32 = 4.0;
const PROPERTY_AXIS_WIDTH: f32 = 12.0;
const PROPERTY_AXIS_GAP: f32 = 4.0;
const PROPERTY_GROUP_GAP: f32 = 6.0;
const PROPERTY_FIELD_INSET_Y: f32 = 3.0;
const PROPERTY_FIELD_RADIUS: f32 = 3.0;
const PROPERTY_FONT_SIZE: f32 = 11.0;
const COMPONENT_PROPERTY_LABEL_WIDTH: f32 = 92.0;
const COMPONENT_PROPERTY_SLOT_03: &str = "WorkbenchComponentPropertySlot03Row";
const COMPONENT_PROPERTY_SLOT_04: &str = "WorkbenchComponentPropertySlot04Row";
const COMPONENT_PROPERTY_VIRTUAL_PREFIX: &str = "WorkbenchComponentPropertyVirtualRow";
const MESH_PROPERTY_ROW: &str = "WorkbenchMeshRow";
const MATERIAL_PROPERTY_ROW: &str = "WorkbenchMaterialRow";

pub(super) fn push_property_row_text_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_property_row(node) {
        return false;
    }

    let label = node.text.trim();
    let value = node.value_text.trim();
    if label.is_empty() && value.is_empty() {
        return false;
    }

    let label_width = property_label_width(node, rect);
    if !label.is_empty() {
        commands.push(text_command(
            FrameRect {
                x: rect.x + PROPERTY_TEXT_INSET_X,
                y: rect.y + PROPERTY_TEXT_INSET_Y,
                width: (label_width - PROPERTY_TEXT_INSET_X * 1.5).max(1.0),
                height: (rect.height - PROPERTY_TEXT_INSET_Y * 2.0).max(1.0),
            },
            clip,
            order,
            label,
            text_color(node),
            opacity,
        ));
    }

    if value.is_empty() {
        return true;
    }

    let value_area = FrameRect {
        x: rect.x + label_width,
        y: rect.y,
        width: (rect.width - label_width - PROPERTY_TEXT_INSET_X).max(1.0),
        height: rect.height,
    };
    let axis_values = property_axis_values(value);
    if axis_values.len() >= 2 {
        push_axis_value_commands(
            commands,
            &axis_values,
            &value_area,
            clip,
            order + 1,
            opacity,
        );
    } else {
        push_scalar_value_commands(commands, clip, node, &value_area, order + 1, value, opacity);
    }
    true
}

fn push_scalar_value_commands(
    commands: &mut Vec<HostPaintCommand>,
    clip: &FrameRect,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    order: i32,
    value: &str,
    opacity: f32,
) {
    let field_rect = FrameRect {
        x: rect.x,
        y: rect.y + PROPERTY_FIELD_INSET_Y,
        width: rect.width,
        height: (rect.height - PROPERTY_FIELD_INSET_Y * 2.0).max(1.0),
    };
    commands.push(HostPaintCommand::quad(
        field_rect.clone(),
        Some(clip.clone()),
        order,
        Some(PALETTE.surface_inset),
        Some(value_field_border_color(node)),
        1.0,
        PROPERTY_FIELD_RADIUS,
        opacity,
    ));
    commands.push(text_command(
        value_text_rect(&field_rect),
        clip,
        order + 1,
        value,
        PALETTE.text,
        opacity,
    ));
}

fn push_axis_value_commands(
    commands: &mut Vec<HostPaintCommand>,
    axis_values: &[PropertyAxisValue],
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let count = axis_values.len().min(4);
    let group_gap_total = PROPERTY_GROUP_GAP * count.saturating_sub(1) as f32;
    let group_width = ((rect.width - group_gap_total) / count as f32).max(1.0);
    let field_height = (rect.height - PROPERTY_FIELD_INSET_Y * 2.0).max(1.0);

    for (index, axis_value) in axis_values.iter().take(count).enumerate() {
        let group_x = rect.x + (group_width + PROPERTY_GROUP_GAP) * index as f32;
        let axis_rect = FrameRect {
            x: group_x,
            y: rect.y + PROPERTY_TEXT_INSET_Y,
            width: PROPERTY_AXIS_WIDTH,
            height: (rect.height - PROPERTY_TEXT_INSET_Y * 2.0).max(1.0),
        };
        commands.push(text_command(
            axis_rect,
            clip,
            order,
            axis_value.axis.as_str(),
            PALETTE.text_muted,
            opacity,
        ));

        let field_rect = FrameRect {
            x: group_x + PROPERTY_AXIS_WIDTH + PROPERTY_AXIS_GAP,
            y: rect.y + PROPERTY_FIELD_INSET_Y,
            width: (group_width - PROPERTY_AXIS_WIDTH - PROPERTY_AXIS_GAP).max(1.0),
            height: field_height,
        };
        commands.push(HostPaintCommand::quad(
            field_rect.clone(),
            Some(clip.clone()),
            order,
            Some(PALETTE.surface_inset),
            Some(PALETTE.border),
            1.0,
            PROPERTY_FIELD_RADIUS,
            opacity,
        ));
        commands.push(text_command(
            value_text_rect(&field_rect),
            clip,
            order + 1,
            axis_value.value.as_str(),
            PALETTE.text,
            opacity,
        ));
    }
}

fn text_command(
    rect: FrameRect,
    clip: &FrameRect,
    order: i32,
    text: &str,
    color: [u8; 4],
    opacity: f32,
) -> HostPaintCommand {
    HostPaintCommand::text(
        rect,
        Some(clip.clone()),
        order,
        text.to_string(),
        color,
        PROPERTY_FONT_SIZE,
        PROPERTY_FONT_SIZE * 1.2,
        UiTextRunPaintStyle::default(),
        opacity,
    )
}

fn value_text_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x + PROPERTY_TEXT_INSET_X,
        y: rect.y + PROPERTY_TEXT_INSET_Y,
        width: (rect.width - PROPERTY_TEXT_INSET_X * 2.0).max(1.0),
        height: (rect.height - PROPERTY_TEXT_INSET_Y * 2.0).max(1.0),
    }
}

fn property_label_width(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    let preferred = if is_component_property_row(node) {
        COMPONENT_PROPERTY_LABEL_WIDTH
    } else {
        PROPERTY_LABEL_WIDTH
    };
    preferred
        .max(PROPERTY_LABEL_MIN_WIDTH)
        .min(rect.width * PROPERTY_LABEL_MAX_WIDTH_RATIO)
        .max(1.0)
}

fn is_property_row(node: &TemplatePaneNodeData) -> bool {
    is_component_property_row(node)
        || node.component_role.as_str() == "property-row"
        || node.role.as_str() == "PropertyRow"
}

fn is_component_property_row(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.control_id.as_str(),
        MESH_PROPERTY_ROW
            | MATERIAL_PROPERTY_ROW
            | COMPONENT_PROPERTY_SLOT_03
            | COMPONENT_PROPERTY_SLOT_04
    ) || node
        .control_id
        .as_str()
        .starts_with(COMPONENT_PROPERTY_VIRTUAL_PREFIX)
}

fn value_field_border_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.focused || node.selected || node.pressed {
        PALETTE.focus_ring
    } else {
        PALETTE.border
    }
}

#[derive(Debug, PartialEq, Eq)]
struct PropertyAxisValue {
    axis: String,
    value: String,
}

fn property_axis_values(value: &str) -> Vec<PropertyAxisValue> {
    let mut values = Vec::new();
    let mut current_axis: Option<String> = None;
    let mut current_value = Vec::new();

    for token in value.split_whitespace() {
        if is_axis_token(token) {
            push_current_axis_value(&mut values, &mut current_axis, &mut current_value);
            current_axis = Some(token.to_string());
        } else if current_axis.is_some() {
            current_value.push(token.to_string());
        }
    }
    push_current_axis_value(&mut values, &mut current_axis, &mut current_value);
    values
}

fn push_current_axis_value(
    values: &mut Vec<PropertyAxisValue>,
    current_axis: &mut Option<String>,
    current_value: &mut Vec<String>,
) {
    let Some(axis) = current_axis.take() else {
        return;
    };
    let value = current_value.join(" ");
    current_value.clear();
    if !value.is_empty() {
        values.push(PropertyAxisValue { axis, value });
    }
}

fn is_axis_token(token: &str) -> bool {
    matches!(token, "X" | "Y" | "Z" | "W")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn property_axis_values_group_units_with_axis_value() {
        assert_eq!(
            property_axis_values("X 0 deg   Y 90 deg   Z -12.5 deg"),
            vec![
                PropertyAxisValue {
                    axis: "X".into(),
                    value: "0 deg".into(),
                },
                PropertyAxisValue {
                    axis: "Y".into(),
                    value: "90 deg".into(),
                },
                PropertyAxisValue {
                    axis: "Z".into(),
                    value: "-12.5 deg".into(),
                },
            ]
        );
    }

    #[test]
    fn component_property_input_rows_use_split_property_row_painter() {
        let node = TemplatePaneNodeData {
            control_id: MESH_PROPERTY_ROW.into(),
            role: "InputField".into(),
            component_role: "input-field".into(),
            text: "Visible".into(),
            value_text: "true".into(),
            ..TemplatePaneNodeData::default()
        };

        assert!(is_property_row(&node));
        assert_eq!(
            property_label_width(
                &node,
                &FrameRect {
                    x: 0.0,
                    y: 0.0,
                    width: 360.0,
                    height: 28.0,
                },
            ),
            COMPONENT_PROPERTY_LABEL_WIDTH
        );
    }
}
