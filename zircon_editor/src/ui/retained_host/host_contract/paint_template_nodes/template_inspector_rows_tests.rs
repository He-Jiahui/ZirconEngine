use super::super::super::data::{FrameRect, TemplateNodeFrameData, TemplatePaneNodeData};
use super::super::template_inspector_row_geometry::{
    chevron_rect, field_rect, nested_label_rect, nested_select_field_rect,
    shadow_check_content_offset_x, shadow_check_rect, INSPECTOR_COUNT_WIDTH, INSPECTOR_LABEL_WIDTH,
};
use super::super::template_inspector_row_kind::{
    inspector_row_kind, InspectorResourceKind, InspectorRowKind, COMPONENT_PROPERTY_SLOT_03,
    MATERIAL_PROPERTY_ROW,
};
use super::super::template_nodes::paint_template_nodes_for_test;
use super::style::{
    disclosure_label_color, resource_chevron_size, resource_count_color, resource_field_background,
    resource_field_border, resource_glyph_color, resource_label_color, resource_value_color,
    INSPECTOR_DISCLOSURE_LABEL_COLOR, RESOURCE_FIELD_BACKGROUND,
};
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
