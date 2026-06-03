use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::template_style::resolved_style_color;
use super::theme::PALETTE;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const INSPECTOR_FONT_SIZE: f32 = 11.0;
const INSPECTOR_ROW_TEXT_Y: f32 = 5.0;
const INSPECTOR_LABEL_WIDTH: f32 = 104.0;
const INSPECTOR_NESTED_LABEL_WIDTH: f32 = 116.0;
const INSPECTOR_NESTED_LABEL_BASE_X: f32 = 6.0;
const INSPECTOR_NESTED_LABEL_OFFSET_X: f32 = 8.0;
const INSPECTOR_NESTED_SELECT_OFFSET_X: f32 = 14.0;
const INSPECTOR_COUNT_WIDTH: f32 = 24.0;
const INSPECTOR_FIELD_INSET_Y: f32 = 3.0;
const INSPECTOR_FIELD_RADIUS: f32 = 4.0;
const INSPECTOR_FIELD_TEXT_X: f32 = 8.0;
const INSPECTOR_FIELD_RIGHT_PAD: f32 = 22.0;
const INSPECTOR_ICON_SIZE: f32 = 13.0;
const INSPECTOR_CHEVRON_SIZE: f32 = 10.0;
const INSPECTOR_CHEVRON_RIGHT_PAD: f32 = 5.0;
const INSPECTOR_SWATCH_SIZE: f32 = 12.0;
const INSPECTOR_CHECK_SIZE: f32 = 14.0;
const INSPECTOR_SHADOW_CHECK_DEFAULT_CONTENT_OFFSET_X: f32 = INSPECTOR_COUNT_WIDTH + 4.0;
const COMPONENT_PROPERTY_SLOT_03: &str = "WorkbenchComponentPropertySlot03Row";
const COMPONENT_PROPERTY_SLOT_04: &str = "WorkbenchComponentPropertySlot04Row";
const COMPONENT_PROPERTY_VIRTUAL_PREFIX: &str = "WorkbenchComponentPropertyVirtualRow";
const MESH_PROPERTY_ROW: &str = "WorkbenchMeshRow";
const MATERIAL_PROPERTY_ROW: &str = "WorkbenchMaterialRow";
const RESOURCE_FIELD_BACKGROUND: [u8; 4] = [22, 28, 32, 255];
const RESOURCE_FIELD_BORDER: [u8; 4] = [40, 50, 56, 255];
const RESOURCE_FIELD_HOVER: [u8; 4] = [31, 40, 45, 255];
const INSPECTOR_LABEL_COLOR: [u8; 4] = [174, 187, 193, 255];
const INSPECTOR_DISCLOSURE_LABEL_COLOR: [u8; 4] = [157, 168, 174, 255];
const INSPECTOR_VALUE_COLOR: [u8; 4] = [198, 210, 215, 255];
const INSPECTOR_COUNT_COLOR: [u8; 4] = [153, 168, 175, 255];
const INSPECTOR_GLYPH_COLOR: [u8; 4] = [148, 165, 173, 255];
const MATERIAL_SWATCH: [u8; 4] = [34, 176, 192, 255];

pub(super) fn push_inspector_row_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    match inspector_row_kind(node) {
        Some(InspectorRowKind::Resource(resource)) => {
            push_resource_row(commands, node, rect, clip, order, resource, opacity);
            true
        }
        Some(InspectorRowKind::Disclosure) => {
            push_disclosure_row(commands, node, rect, clip, order, opacity);
            true
        }
        Some(InspectorRowKind::ShadowSelect) => {
            push_shadow_select_row(commands, node, rect, clip, order, opacity);
            true
        }
        Some(InspectorRowKind::ShadowCheck) => {
            push_shadow_check_row(commands, node, rect, clip, order, opacity);
            true
        }
        None => false,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InspectorRowKind {
    Resource(InspectorResourceKind),
    Disclosure,
    ShadowSelect,
    ShadowCheck,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InspectorResourceKind {
    Mesh,
    Material,
}

fn inspector_row_kind(node: &TemplatePaneNodeData) -> Option<InspectorRowKind> {
    if !is_inspector_property_row(node) {
        return None;
    }

    let label = node.text.trim();
    let value = node.value_text.trim();
    if label.eq_ignore_ascii_case("Lighting") && value.is_empty() {
        return Some(InspectorRowKind::Disclosure);
    }
    if label.eq_ignore_ascii_case("Mesh") && !value.is_empty() {
        return Some(InspectorRowKind::Resource(InspectorResourceKind::Mesh));
    }
    if matches_ignore_ascii_case(label, &["Material", "Materials"]) && !value.is_empty() {
        return Some(InspectorRowKind::Resource(InspectorResourceKind::Material));
    }
    if label.eq_ignore_ascii_case("Cast Shadows") && !value.is_empty() {
        return Some(InspectorRowKind::ShadowSelect);
    }
    if label.eq_ignore_ascii_case("Receive Shadows") && !value.is_empty() {
        return Some(InspectorRowKind::ShadowCheck);
    }
    None
}

fn is_inspector_property_row(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.control_id.as_str(),
        MESH_PROPERTY_ROW
            | MATERIAL_PROPERTY_ROW
            | COMPONENT_PROPERTY_SLOT_03
            | COMPONENT_PROPERTY_SLOT_04
            | "WorkbenchInspectorLightingRow"
    ) || node
        .control_id
        .as_str()
        .starts_with(COMPONENT_PROPERTY_VIRTUAL_PREFIX)
}

fn push_resource_row(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    resource: InspectorResourceKind,
    opacity: f32,
) {
    let count_width = if resource == InspectorResourceKind::Material {
        INSPECTOR_COUNT_WIDTH
    } else {
        0.0
    };
    push_label(
        commands,
        rect,
        clip,
        order,
        node.text.trim(),
        resource_label_color(node),
        opacity,
    );
    if resource == InspectorResourceKind::Material {
        push_text(
            commands,
            FrameRect {
                x: rect.x + INSPECTOR_LABEL_WIDTH,
                y: rect.y + INSPECTOR_ROW_TEXT_Y,
                width: INSPECTOR_COUNT_WIDTH,
                height: (rect.height - INSPECTOR_ROW_TEXT_Y * 2.0).max(1.0),
            },
            clip,
            order + 1,
            "1",
            resource_count_color(node),
            opacity,
        );
    }

    let field = field_rect(
        rect,
        INSPECTOR_LABEL_WIDTH + count_width,
        rect.width - INSPECTOR_LABEL_WIDTH - count_width,
    );
    push_field(commands, node, &field, clip, order + 2, opacity);

    let leading = leading_affordance_rect(&field);
    let glyph_color = resource_glyph_color(node);
    match resource {
        InspectorResourceKind::Mesh => {
            push_cube_icon(commands, &leading, clip, order + 3, glyph_color, opacity)
        }
        InspectorResourceKind::Material => {
            push_swatch(commands, &leading, clip, order + 3, opacity)
        }
    }

    let text_x = leading.x + leading.width + 7.0;
    push_text(
        commands,
        FrameRect {
            x: text_x,
            y: field.y + 5.0,
            width: (field.x + field.width - text_x - INSPECTOR_FIELD_RIGHT_PAD).max(1.0),
            height: (field.height - 10.0).max(1.0),
        },
        clip,
        order + 4,
        node.value_text.trim(),
        resource_value_color(node),
        opacity,
    );
    let chevron_size = resource_chevron_size(node);
    push_down_chevron(
        commands,
        &chevron_rect(&field, chevron_size),
        clip,
        order + 5,
        glyph_color,
        opacity,
    );
}

fn resource_value_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    declared_color(node.value_color).unwrap_or(INSPECTOR_VALUE_COLOR)
}

fn resource_label_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    declared_color(node.label_color).unwrap_or(INSPECTOR_LABEL_COLOR)
}

fn resource_count_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    declared_color(node.label_color).unwrap_or(INSPECTOR_COUNT_COLOR)
}

fn resource_glyph_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    declared_color(node.icon_color).unwrap_or(INSPECTOR_GLYPH_COLOR)
}

fn resource_chevron_size(node: &TemplatePaneNodeData) -> f32 {
    let size = node.layout_icon_size;
    if size.is_finite() && size > 0.0 {
        size
    } else {
        INSPECTOR_CHEVRON_SIZE
    }
}

fn push_disclosure_row(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let chevron = FrameRect {
        x: rect.x + 2.0,
        y: rect.y + (rect.height - 12.0).max(0.0) * 0.5,
        width: 12.0,
        height: 12.0,
    };
    push_down_chevron(
        commands,
        &chevron,
        clip,
        order,
        INSPECTOR_GLYPH_COLOR,
        opacity,
    );
    push_text(
        commands,
        FrameRect {
            x: chevron.x + chevron.width + 5.0,
            y: rect.y + INSPECTOR_ROW_TEXT_Y,
            width: (rect.width - chevron.width - 12.0).max(1.0),
            height: (rect.height - INSPECTOR_ROW_TEXT_Y * 2.0).max(1.0),
        },
        clip,
        order + 1,
        node.text.trim(),
        disclosure_label_color(),
        opacity,
    );
}

fn disclosure_label_color() -> [u8; 4] {
    INSPECTOR_DISCLOSURE_LABEL_COLOR
}

fn push_shadow_select_row(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_nested_label(commands, rect, clip, order, node.text.trim(), opacity);
    let field = nested_select_field_rect(rect);
    push_field(commands, node, &field, clip, order + 1, opacity);
    let value = bool_display_value(node.value_text.trim());
    push_text(
        commands,
        FrameRect {
            x: field.x + INSPECTOR_FIELD_TEXT_X,
            y: field.y + 5.0,
            width: (field.width - INSPECTOR_FIELD_TEXT_X - INSPECTOR_FIELD_RIGHT_PAD).max(1.0),
            height: (field.height - 10.0).max(1.0),
        },
        clip,
        order + 2,
        value,
        resource_value_color(node),
        opacity,
    );
    push_down_chevron(
        commands,
        &chevron_rect(&field, INSPECTOR_CHEVRON_SIZE),
        clip,
        order + 3,
        INSPECTOR_GLYPH_COLOR,
        opacity,
    );
}

fn push_shadow_check_row(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_nested_label(commands, rect, clip, order, node.text.trim(), opacity);
    let check = shadow_check_rect(node, rect);
    let checked = bool_value(node.value_text.trim()) || node.checked || node.selected;
    commands.push(HostPaintCommand::quad(
        check.clone(),
        Some(clip.clone()),
        order + 1,
        Some(if checked {
            PALETTE.accent_soft
        } else {
            PALETTE.surface_inset
        }),
        Some(if checked {
            PALETTE.accent
        } else {
            RESOURCE_FIELD_BORDER
        }),
        1.0,
        3.0,
        opacity,
    ));
    if checked {
        push_check_tick(commands, &check, clip, order + 2, opacity);
    }
}

fn shadow_check_rect(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x + INSPECTOR_NESTED_LABEL_WIDTH + shadow_check_content_offset_x(node),
        y: rect.y + (rect.height - INSPECTOR_CHECK_SIZE).max(0.0) * 0.5,
        width: INSPECTOR_CHECK_SIZE,
        height: INSPECTOR_CHECK_SIZE,
    }
}

fn shadow_check_content_offset_x(node: &TemplatePaneNodeData) -> f32 {
    let declared_offset = node.layout_content_offset_x;
    if declared_offset.is_finite() && declared_offset > 0.0 {
        declared_offset
    } else {
        INSPECTOR_SHADOW_CHECK_DEFAULT_CONTENT_OFFSET_X
    }
}

fn push_label(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    label: &str,
    color: [u8; 4],
    opacity: f32,
) {
    push_text(
        commands,
        FrameRect {
            x: rect.x + 1.0,
            y: rect.y + INSPECTOR_ROW_TEXT_Y,
            width: INSPECTOR_LABEL_WIDTH - 4.0,
            height: (rect.height - INSPECTOR_ROW_TEXT_Y * 2.0).max(1.0),
        },
        clip,
        order,
        label,
        color,
        opacity,
    );
}

fn push_nested_label(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    label: &str,
    opacity: f32,
) {
    push_text(
        commands,
        nested_label_rect(rect),
        clip,
        order,
        label,
        INSPECTOR_LABEL_COLOR,
        opacity,
    );
}

fn nested_label_rect(rect: &FrameRect) -> FrameRect {
    let x = rect.x + INSPECTOR_NESTED_LABEL_BASE_X + INSPECTOR_NESTED_LABEL_OFFSET_X;
    FrameRect {
        x,
        y: rect.y + INSPECTOR_ROW_TEXT_Y,
        width: (INSPECTOR_NESTED_LABEL_WIDTH - (x - rect.x) - 4.0).max(1.0),
        height: (rect.height - INSPECTOR_ROW_TEXT_Y * 2.0).max(1.0),
    }
}

fn nested_select_field_rect(rect: &FrameRect) -> FrameRect {
    let left_offset =
        INSPECTOR_NESTED_LABEL_WIDTH + INSPECTOR_COUNT_WIDTH + INSPECTOR_NESTED_SELECT_OFFSET_X;
    field_rect(rect, left_offset, rect.width - left_offset)
}

fn push_field(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(resource_field_background(node)),
        Some(if node.focused {
            PALETTE.focus_ring
        } else {
            resource_field_border(node)
        }),
        1.0,
        INSPECTOR_FIELD_RADIUS,
        opacity,
    ));
}

fn resource_field_background(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.hovered || node.pressed {
        RESOURCE_FIELD_HOVER
    } else {
        resolved_style_color(node.button_style.element.background_color.as_ref())
            .filter(|color| color[3] > 0)
            .unwrap_or(RESOURCE_FIELD_BACKGROUND)
    }
}

fn resource_field_border(node: &TemplatePaneNodeData) -> [u8; 4] {
    resolved_style_color(node.button_style.element.border_color.as_ref())
        .filter(|color| color[3] > 0)
        .unwrap_or(RESOURCE_FIELD_BORDER)
}

fn push_text(
    commands: &mut Vec<HostPaintCommand>,
    rect: FrameRect,
    clip: &FrameRect,
    order: i32,
    text: &str,
    color: [u8; 4],
    opacity: f32,
) {
    if text.trim().is_empty() {
        return;
    }
    commands.push(HostPaintCommand::text(
        rect,
        Some(clip.clone()),
        order,
        text.to_string(),
        color,
        INSPECTOR_FONT_SIZE,
        INSPECTOR_FONT_SIZE * 1.2,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn field_rect(rect: &FrameRect, left_offset: f32, width: f32) -> FrameRect {
    FrameRect {
        x: rect.x + left_offset,
        y: rect.y + INSPECTOR_FIELD_INSET_Y,
        width: width.max(1.0),
        height: (rect.height - INSPECTOR_FIELD_INSET_Y * 2.0).max(1.0),
    }
}

fn leading_affordance_rect(field: &FrameRect) -> FrameRect {
    FrameRect {
        x: field.x + 8.0,
        y: field.y + (field.height - INSPECTOR_ICON_SIZE).max(0.0) * 0.5,
        width: INSPECTOR_ICON_SIZE,
        height: INSPECTOR_ICON_SIZE,
    }
}

fn chevron_rect(field: &FrameRect, size: f32) -> FrameRect {
    let size = if size.is_finite() && size > 0.0 {
        size
    } else {
        INSPECTOR_CHEVRON_SIZE
    };
    FrameRect {
        x: field.x + field.width - size - INSPECTOR_CHEVRON_RIGHT_PAD,
        y: field.y + (field.height - size).max(0.0) * 0.5,
        width: size,
        height: size,
    }
}

fn push_swatch(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let swatch = FrameRect {
        x: rect.x + (rect.width - INSPECTOR_SWATCH_SIZE).max(0.0) * 0.5,
        y: rect.y + (rect.height - INSPECTOR_SWATCH_SIZE).max(0.0) * 0.5,
        width: INSPECTOR_SWATCH_SIZE,
        height: INSPECTOR_SWATCH_SIZE,
    };
    commands.push(HostPaintCommand::quad(
        swatch,
        Some(clip.clone()),
        order,
        Some(MATERIAL_SWATCH),
        Some([21, 95, 105, 255]),
        1.0,
        INSPECTOR_SWATCH_SIZE * 0.5,
        opacity,
    ));
}

fn push_cube_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    for part in [
        FrameRect {
            x: rect.x + 3.0,
            y: rect.y + 3.0,
            width: rect.width - 6.0,
            height: rect.height - 6.0,
        },
        FrameRect {
            x: rect.x + 5.0,
            y: rect.y + 1.0,
            width: rect.width - 6.0,
            height: 2.0,
        },
        FrameRect {
            x: rect.x + rect.width - 3.0,
            y: rect.y + 4.0,
            width: 2.0,
            height: rect.height - 7.0,
        },
    ] {
        commands.push(HostPaintCommand::quad(
            part,
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            1.0,
            opacity,
        ));
    }
}

fn push_down_chevron(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    let parts = if rect.width >= 14.0 && rect.height >= 14.0 {
        let block = 3.0;
        let center_x = rect.x + rect.width * 0.5;
        let center_y = rect.y + rect.height * 0.5;
        [
            FrameRect {
                x: center_x - block * 1.5,
                y: center_y - block,
                width: block,
                height: block,
            },
            FrameRect {
                x: center_x - block * 0.5,
                y: center_y,
                width: block,
                height: block,
            },
            FrameRect {
                x: center_x + block * 0.5,
                y: center_y - block,
                width: block,
                height: block,
            },
        ]
    } else {
        [
            FrameRect {
                x: rect.x + 2.0,
                y: rect.y + 3.0,
                width: 2.0,
                height: 2.0,
            },
            FrameRect {
                x: rect.x + 4.0,
                y: rect.y + 5.0,
                width: 2.0,
                height: 2.0,
            },
            FrameRect {
                x: rect.x + 6.0,
                y: rect.y + 3.0,
                width: 2.0,
                height: 2.0,
            },
        ]
    };
    for part in parts {
        commands.push(HostPaintCommand::quad(
            part,
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }
}

fn push_check_tick(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    for part in [
        FrameRect {
            x: rect.x + 3.0,
            y: rect.y + 7.0,
            width: 3.0,
            height: 3.0,
        },
        FrameRect {
            x: rect.x + 5.0,
            y: rect.y + 9.0,
            width: 3.0,
            height: 3.0,
        },
        FrameRect {
            x: rect.x + 8.0,
            y: rect.y + 4.0,
            width: 3.0,
            height: 8.0,
        },
    ] {
        commands.push(HostPaintCommand::quad(
            part,
            Some(clip.clone()),
            order,
            Some(PALETTE.shell_background),
            None,
            0.0,
            1.0,
            opacity,
        ));
    }
}

fn bool_display_value(value: &str) -> &'static str {
    if bool_value(value) {
        "On"
    } else {
        "Off"
    }
}

fn bool_value(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "true" | "1" | "on" | "yes" | "check" | "checked"
    )
}

fn matches_ignore_ascii_case(value: &str, candidates: &[&str]) -> bool {
    candidates
        .iter()
        .any(|candidate| value.eq_ignore_ascii_case(candidate))
}

fn declared_color(color: crate::ui::retained_host::primitives::Color) -> Option<[u8; 4]> {
    (color.a > 0).then_some([color.r, color.g, color.b, color.a])
}

#[cfg(test)]
mod tests {
    use super::super::super::data::TemplateNodeFrameData;
    use super::super::template_nodes::paint_template_nodes_for_test;
    use super::*;
    use crate::ui::layouts::common::model_rc;

    #[test]
    fn inspector_row_kind_only_promotes_known_resource_and_shadow_rows() {
        assert_eq!(
            inspector_row_kind(&inspector_node("WorkbenchMeshRow", "Mesh", "Box_01")),
            Some(InspectorRowKind::Resource(InspectorResourceKind::Mesh))
        );
        assert_eq!(
            inspector_row_kind(&inspector_node(
                "WorkbenchMaterialRow",
                "Cast Shadows",
                "false"
            )),
            Some(InspectorRowKind::ShadowSelect)
        );
        assert_eq!(
            inspector_row_kind(&inspector_node("WorkbenchMeshRow", "Visible", "true")),
            None
        );
    }

    #[test]
    fn mesh_resource_row_paints_field_icon_and_chevron() {
        let bytes = paint_template_nodes_for_test(
            320,
            48,
            model_rc(vec![inspector_node("WorkbenchMeshRow", "Mesh", "Box_01")]),
        );

        assert_eq!(pixel_at(&bytes, 320, 136, 20), RESOURCE_FIELD_BACKGROUND);
        assert!(changed_pixel_count(&bytes, 320, 114, 15, 14, 12) > 0);
        assert!(changed_pixel_count(&bytes, 320, 300, 16, 12, 10) > 0);
    }

    #[test]
    fn material_resource_row_paints_count_and_swatch() {
        let bytes = paint_template_nodes_for_test(
            320,
            48,
            model_rc(vec![inspector_node(
                "WorkbenchMaterialRow",
                "Materials",
                "M_Metal",
            )]),
        );

        assert!(changed_pixel_count(&bytes, 320, 104, 12, 16, 18) > 0);
        assert!(changed_pixel_count(&bytes, 320, 136, 16, 14, 14) > 0);
    }

    #[test]
    fn resource_row_style_uses_declared_value_and_chevron_fields() {
        let mut node = inspector_node("WorkbenchMaterialRow", "Materials", "M_Metal");
        node.label_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(154, 165, 171);
        node.value_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(143, 154, 160);
        node.icon_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(194, 204, 209);
        node.button_style = resolved_background_and_border([19, 24, 27, 255], [32, 39, 44, 255]);
        node.layout_icon_size = 15.0;

        assert_eq!(resource_label_color(&node), [154, 165, 171, 255]);
        assert_eq!(resource_count_color(&node), [154, 165, 171, 255]);
        assert_eq!(resource_value_color(&node), [143, 154, 160, 255]);
        assert_eq!(resource_glyph_color(&node), [194, 204, 209, 255]);
        assert_eq!(resource_field_background(&node), [19, 24, 27, 255]);
        assert_eq!(resource_field_border(&node), [32, 39, 44, 255]);
        assert_eq!(resource_chevron_size(&node), 15.0);

        let frame = FrameRect {
            x: node.frame.x,
            y: node.frame.y,
            width: node.frame.width,
            height: node.frame.height,
        };
        let field = field_rect(
            &frame,
            INSPECTOR_LABEL_WIDTH + INSPECTOR_COUNT_WIDTH,
            node.frame.width - INSPECTOR_LABEL_WIDTH - INSPECTOR_COUNT_WIDTH,
        );
        let chevron = chevron_rect(&field, resource_chevron_size(&node));
        assert_eq!(chevron.width, 15.0);
        assert_eq!(chevron.height, 15.0);
        assert!((chevron.x - (field.x + field.width - 20.0)).abs() < 0.001);
    }

    fn resolved_background_and_border(
        background: [u8; 4],
        border: [u8; 4],
    ) -> zircon_runtime_interface::ui::style::ResolvedButtonStyle {
        use zircon_runtime_interface::ui::style::{
            ResolvedButtonStyle, UiResolvedElementStyle, UiRgbaColor, UiStyleColor,
        };

        ResolvedButtonStyle {
            element: UiResolvedElementStyle {
                background_color: Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(
                    background[0],
                    background[1],
                    background[2],
                    background[3],
                ))),
                border_color: Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(
                    border[0], border[1], border[2], border[3],
                ))),
                ..UiResolvedElementStyle::default()
            },
            ..ResolvedButtonStyle::default()
        }
    }

    #[test]
    fn receive_shadows_row_paints_checked_box_without_full_field() {
        let mut row = inspector_node(COMPONENT_PROPERTY_SLOT_03, "Receive Shadows", "true");
        row.layout_content_offset_x = 34.0;
        let bytes = paint_template_nodes_for_test(320, 48, model_rc(vec![row.clone()]));
        let rect = FrameRect {
            x: row.frame.x,
            y: row.frame.y,
            width: row.frame.width,
            height: row.frame.height,
        };

        assert_eq!(shadow_check_content_offset_x(&row), 34.0);
        assert_eq!(shadow_check_rect(&row, &rect).x, 158.0);
        assert!(changed_pixel_count(&bytes, 320, 156, 14, 20, 18) > 0);
        assert_eq!(pixel_at(&bytes, 320, 250, 16), [0, 0, 0, 255]);
    }

    #[test]
    fn nested_lighting_select_preserves_right_edge_with_select_indent() {
        let rect = FrameRect {
            x: 8.0,
            y: 8.0,
            width: 304.0,
            height: 28.0,
        };

        let label = nested_label_rect(&rect);
        let field = nested_select_field_rect(&rect);

        assert_eq!(label.x, 22.0);
        assert_eq!(field.x, 162.0);
        assert_eq!(field.width, 150.0);
        assert_eq!(field.x + field.width, rect.x + rect.width);
    }

    #[test]
    fn cast_shadows_select_uses_declared_field_and_value_tones() {
        let mut row = inspector_node(MATERIAL_PROPERTY_ROW, "Cast Shadows", "false");
        row.button_style = resolved_background_and_border([40, 46, 50, 255], [52, 61, 67, 255]);
        row.value_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(181, 192, 197);

        assert_eq!(resource_field_background(&row), [40, 46, 50, 255]);
        assert_eq!(resource_field_border(&row), [52, 61, 67, 255]);
        assert_eq!(resource_value_color(&row), [181, 192, 197, 255]);
    }

    #[test]
    fn receive_shadows_row_uses_legacy_checkbox_offset_without_declaration() {
        let row = inspector_node(COMPONENT_PROPERTY_SLOT_03, "Receive Shadows", "true");
        let rect = FrameRect {
            x: row.frame.x,
            y: row.frame.y,
            width: row.frame.width,
            height: row.frame.height,
        };

        assert_eq!(shadow_check_content_offset_x(&row), 28.0);
        assert_eq!(shadow_check_rect(&row, &rect).x, 152.0);
    }

    #[test]
    fn lighting_disclosure_row_paints_chevron_and_label_only() {
        let bytes = paint_template_nodes_for_test(
            220,
            42,
            model_rc(vec![inspector_node(
                "WorkbenchInspectorLightingRow",
                "Lighting",
                "",
            )]),
        );

        assert!(changed_pixel_count(&bytes, 220, 2, 12, 16, 16) > 0);
        assert_eq!(changed_pixel_count(&bytes, 220, 150, 10, 50, 20), 0);
        assert_eq!(disclosure_label_color(), INSPECTOR_DISCLOSURE_LABEL_COLOR);
    }

    fn inspector_node(control_id: &str, label: &str, value: &str) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: control_id.into(),
            role: "InputField".into(),
            component_role: "input-field".into(),
            text: label.into(),
            value_text: value.into(),
            frame: TemplateNodeFrameData {
                x: 8.0,
                y: 8.0,
                width: 304.0,
                height: 28.0,
            },
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
