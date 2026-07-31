use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::super::push_inspector_row_commands;
use super::super::style::{inspector_row_palette, resource_label_color};
use super::support::{changed_pixel_count, inspector_node};
use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::host_contract::data::FrameRect;

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
    assert_eq!(
        resource_label_color(&inspector_node(
            "WorkbenchInspectorLightingRow",
            "Lighting",
            ""
        )),
        inspector_row_palette().label
    );
}

#[test]
fn disclosure_row_paints_shell_dropdown_asset_pixels() {
    let node = inspector_node("WorkbenchInspectorLightingRow", "Lighting", "");
    let rect = FrameRect {
        x: node.frame.x,
        y: node.frame.y,
        width: node.frame.width,
        height: node.frame.height,
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

    let assets = commands
        .iter()
        .filter_map(|command| command.image_pixels.as_ref())
        .collect::<Vec<_>>();
    assert!(
        !assets.is_empty(),
        "disclosure row should render its chevron through shell icon pixels"
    );
    assert!(
        assets
            .iter()
            .all(|image| !image.resource_key.starts_with("missing-icon:")),
        "disclosure row should not use missing-icon pixels"
    );
}

#[test]
fn disclosure_row_uses_its_declared_label_color() {
    let mut node = inspector_node("WorkbenchInspectorLightingRow", "Lighting", "");
    node.label_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(118, 129, 137);
    let rect = FrameRect {
        x: node.frame.x,
        y: node.frame.y,
        width: node.frame.width,
        height: node.frame.height,
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

    assert!(commands.iter().any(|command| {
        command.text.as_deref() == Some("Lighting")
            && command.foreground_color == Some([118, 129, 137, 255])
    }));
}
