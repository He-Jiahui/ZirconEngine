use super::*;

#[test]
fn blend_space_details_composes_shared_property_editor_rows() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let details = std::fs::read_to_string(manifest.join(
        "assets/ui/editor/components/workbench/composites/animation/\
             workbench_blend_space_details.zui",
    ))
    .expect("Blend Space details composite should be readable");
    let property_editor_row = std::fs::read_to_string(manifest.join(
        "assets/ui/editor/components/workbench/composites/inputs/\
             workbench_property_editor_row.zui",
    ))
    .expect("shared property editor row should be readable");

    for required in [
        "[components.WorkbenchPropertyEditorRow]",
        "component = \"PropertyRow\"",
        "slots = { value = { multiple = false } }",
        "props = { name = \"value\" }",
    ] {
        assert!(
            property_editor_row.contains(required),
            "missing shared property-editor row contract: {required}"
        );
    }
    for required in [
        "workbench_property_editor_row.zui#WorkbenchPropertyEditorRow",
        "component = \"WorkbenchPropertyEditorRow\"",
        "control_id = \"WorkbenchExtensionBlendSpaceAssetPropertyRow\"",
        "control_id = \"WorkbenchExtensionBlendSpaceXAxisPropertyRow\"",
        "control_id = \"WorkbenchExtensionBlendSpaceInterpolationPropertyRow\"",
        "text = \"Asset\"",
        "text = \"X Axis\"",
        "text = \"Interpolation\"",
        "node = \"asset\", slot = { name = \"value\" }",
        "node = \"x_axis\", slot = { name = \"value\" }",
        "node = \"interpolation\", slot = { name = \"value\" }",
    ] {
        assert!(
            details.contains(required),
            "Blend Space details must consume shared name/value property rows: {required}"
        );
    }
    assert!(
        !details.contains("value = \"X Axis: Speed 0-620\""),
        "the editable value slot must not duplicate its label inside field text"
    );
    for forbidden in [
        "background_color =",
        "border_color =",
        "foreground_color =",
        "font_size =",
        "font_weight =",
    ] {
        assert!(
            !property_editor_row.contains(forbidden),
            "shared property editor row must inherit visual tokens: {forbidden}"
        );
    }
}

#[test]
fn blend_space_property_rows_project_values_and_bound_editors_to_value_column() {
    let bridge = open_blend_space_bridge(1260, 780);
    let projection = bridge.host_projection();

    for (control_id, label, value) in [
        (
            "WorkbenchExtensionBlendSpaceXAxisProperty",
            "Horizontal",
            "Speed  0 - 620",
        ),
        (
            "WorkbenchExtensionBlendSpaceYAxisProperty",
            "Vertical",
            "Direction  -180 - 180",
        ),
        (
            "WorkbenchExtensionBlendSpaceGridDivisionsProperty",
            "Grid divisions",
            "4 x 4",
        ),
        (
            "WorkbenchExtensionBlendSpaceSamplePositionProperty",
            "Sample position",
            "420, 0",
        ),
        (
            "WorkbenchExtensionBlendSpaceSampleRateProperty",
            "Rate scale",
            "1.00",
        ),
    ] {
        let node = projection
            .nodes
            .iter()
            .find(|node| node.control_id.as_deref() == Some(control_id))
            .unwrap_or_else(|| panic!("{control_id} should exist in the retained projection"));
        assert_eq!(node.text.as_deref(), Some(label));
        assert_eq!(node.value_text.as_deref(), Some(value));
    }

    for (row_id, editor_id) in [
        (
            "WorkbenchExtensionBlendSpaceAssetPropertyRow",
            "WorkbenchExtensionBlendSpaceAssetDropdown",
        ),
        (
            "WorkbenchExtensionBlendSpaceXAxisPropertyRow",
            "WorkbenchExtensionBlendSpaceXAxisField",
        ),
        (
            "WorkbenchExtensionBlendSpaceInterpolationPropertyRow",
            "WorkbenchExtensionBlendSpaceInterpolationDropdown",
        ),
    ] {
        let row = required_frame(&bridge, row_id);
        let editor = required_frame(&bridge, editor_id);
        eprintln!("property-editor-row-frame row={row_id} {row:?} editor={editor_id} {editor:?}");
        assert!(
            editor.x >= row.x + 60.0,
            "{editor_id} must start after the shared name column"
        );
        assert!(editor.x + editor.width <= row.x + row.width + 0.5);
        assert!(
            editor.width >= row.width * 0.40,
            "{editor_id} must stretch through the shared value column"
        );
        assert!(editor.height <= row.height + 0.5);
    }
}
