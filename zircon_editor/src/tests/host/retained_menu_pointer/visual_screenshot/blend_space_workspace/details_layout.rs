use super::support::*;
use super::*;

#[test]
fn blend_space_wide_details_include_dense_sample_rows() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = std::fs::read_to_string(manifest.join(
        "assets/ui/editor/components/workbench/modules/extensions/animation/\
             workbench_extension_blend_space_workspace.zui",
    ))
    .expect("Blend Space workspace asset should be readable");
    let details = std::fs::read_to_string(manifest.join(
        "assets/ui/editor/components/workbench/composites/animation/\
             workbench_blend_space_details.zui",
    ))
    .expect("shared Blend Space details composite should be readable");

    for required in [
        "[components.WorkbenchBlendSpaceDetails]",
        "component = \"WorkbenchSectionTitle\"",
        "control_id = \"WorkbenchExtensionBlendSpaceSamplesTitle\"",
        "text = \"SAMPLES (8)\"",
        "control_id = \"WorkbenchExtensionBlendSpaceSampleRunForwardRow\"",
        "options = [\"Run_Fwd\", \"0\", \"600\", \"1.00\"]",
        "control_id = \"WorkbenchExtensionBlendSpaceSampleStrafeLeftRow\"",
        "control_id = \"WorkbenchExtensionBlendSpaceSampleStrafeRightRow\"",
        "control_id = \"WorkbenchExtensionBlendSpaceSampleIdleRow\"",
        "control_id = \"WorkbenchExtensionBlendSpaceAxisGroupTitle\"",
        "control_id = \"WorkbenchExtensionBlendSpaceSampleDetailTitle\"",
    ] {
        assert!(
            details.contains(required),
            "missing reference-density sample table contract: {required}"
        );
    }
    for required in [
        "workbench_blend_space_details.zui#WorkbenchBlendSpaceDetails",
        "component = \"WorkbenchBlendSpaceDetails\"",
        "control_id = \"WorkbenchExtensionBlendSpaceDetails\"",
    ] {
        assert!(
            source.contains(required),
            "wide workspace must compose the shared Details asset: {required}"
        );
    }
    for preserved_route in [
        "workbench.extension.blend_space.run_sample_table_row.select",
        "workbench.extension.blend_space.walk_sample_table_row.select",
        "workbench.extension.blend_space.diagonal_sample_table_row.select",
        "workbench.extension.blend_space.idle_sample_table_row.select",
        "workbench.extension.blend_space.asset.edit",
        "workbench.extension.blend_space.asset.commit",
        "workbench.extension.blend_space.x_axis.edit",
        "workbench.extension.blend_space.x_axis.commit",
        "workbench.extension.blend_space.interpolation.edit",
        "workbench.extension.blend_space.interpolation.commit",
    ] {
        assert!(
            details.contains(preserved_route),
            "Details extraction must preserve the authored route: {preserved_route}"
        );
    }
    for deprecated_route in [
        "workbench.extension.blend_space.sample.run",
        "workbench.extension.blend_space.sample.walk",
        "workbench.extension.blend_space.sample.diagonal",
        "workbench.extension.blend_space.sample.idle",
    ] {
        assert!(
            !details.contains(deprecated_route),
            "Details extraction must use its canonical binding action, not {deprecated_route}"
        );
    }
    for forbidden in [
        "background_color =",
        "border_color =",
        "foreground_color =",
        "font_size =",
        "font_weight =",
    ] {
        assert!(
            !details.contains(forbidden),
            "Details composite must inherit shared primitive visuals: {forbidden}"
        );
    }
}
