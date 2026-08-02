use super::super::super::super::data::FrameRect;
use super::super::super::template_inspector_row_geometry::{
    INSPECTOR_COUNT_WIDTH, INSPECTOR_LABEL_WIDTH, chevron_rect, field_rect,
};
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::super::push_inspector_row_commands;
use super::super::style::{
    resource_chevron_size, resource_count_color, resource_field_background, resource_field_border,
    resource_glyph_color, resource_label_color, resource_value_color,
};
use super::support::{
    changed_pixel_count, inspector_node, pixel_at, resolved_background_and_border,
};
use crate::ui::layouts::common::model_rc;

#[test]
fn mesh_resource_row_paints_field_icon_and_chevron() {
    let node = inspector_node("WorkbenchMeshRow", "Mesh", "Box_01");
    let bytes = paint_template_nodes_for_test(320, 48, model_rc(vec![node.clone()]));

    assert_eq!(
        pixel_at(&bytes, 320, 136, 20),
        resource_field_background(&node)
    );
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

#[test]
fn resource_rows_paint_shell_asset_pixels_for_leading_icon_and_chevron() {
    let rect = FrameRect {
        x: 8.0,
        y: 8.0,
        width: 304.0,
        height: 28.0,
    };

    for node in [
        inspector_node("WorkbenchMeshRow", "Mesh", "Box_01"),
        inspector_node("WorkbenchMaterialRow", "Materials", "M_Metal"),
    ] {
        let mut commands = Vec::new();
        assert!(push_inspector_row_commands(
            &mut commands,
            &node,
            &rect,
            &rect,
            0,
            1.0,
        ));

        let assets = commands
            .iter()
            .filter_map(|command| command.image_pixels.as_ref())
            .collect::<Vec<_>>();
        assert!(
            assets.len() >= 2,
            "{} should render a leading resource icon and dropdown asset",
            node.control_id
        );
        assert!(
            assets
                .iter()
                .all(|image| !image.resource_key.starts_with("missing-icon:")),
            "{} should not use missing-icon pixels",
            node.control_id
        );
    }
}

#[test]
fn material_resource_row_does_not_emit_an_oversized_swatch_for_a_tiny_field() {
    let node = inspector_node("WorkbenchMaterialRow", "Materials", "M_Metal");
    let rect = FrameRect {
        x: 8.0,
        y: 8.0,
        width: 129.0,
        height: 7.0,
    };
    let mut commands = Vec::new();

    assert!(push_inspector_row_commands(
        &mut commands,
        &node,
        &rect,
        &rect,
        0,
        1.0,
    ));

    assert!(commands.iter().all(|command| command.z_index != 3));
}
