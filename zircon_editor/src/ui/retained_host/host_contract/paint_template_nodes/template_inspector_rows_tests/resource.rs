use super::super::super::super::data::FrameRect;
use super::super::super::template_inspector_row_geometry::{
    chevron_rect, field_rect, INSPECTOR_COUNT_WIDTH, INSPECTOR_LABEL_WIDTH,
};
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::super::style::{
    resource_chevron_size, resource_count_color, resource_field_background, resource_field_border,
    resource_glyph_color, resource_label_color, resource_value_color, RESOURCE_FIELD_BACKGROUND,
};
use super::support::{
    changed_pixel_count, inspector_node, pixel_at, resolved_background_and_border,
};
use crate::ui::layouts::common::model_rc;

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
