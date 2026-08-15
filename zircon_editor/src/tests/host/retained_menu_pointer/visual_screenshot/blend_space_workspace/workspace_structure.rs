use super::support::*;
use super::*;

#[test]
fn workbench_caption_owns_unreal_compact_typography_contract() {
    let runtime_caption_size =
        zircon_runtime_interface::ui::design_tokens::EditorDesignTokens::workbench_dark()
            .typography
            .caption_size;
    assert!(
        (runtime_caption_size - 10.666667).abs() <= 0.000_01,
        "Workbench caption must project the Runtime text caption metric: {runtime_caption_size}"
    );

    let source = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets/ui/editor/components/workbench/primitives/data/workbench_caption.zui"),
    )
    .expect("Workbench caption primitive should be readable");

    for required in [
        "[components.WorkbenchCaption]",
        "component = \"Label\"",
        "font_size = \"$editor.typography.caption.size\"",
        "font_weight = \"$editor.typography.strong.weight\"",
        "text_tone = \"secondary\"",
        "height = { min = 18.0, preferred = 20.0, max = 22.0, stretch = \"Fixed\" }",
    ] {
        assert!(
            source.contains(required),
            "missing shared Unreal compact-caption contract: {required}"
        );
    }
    assert!(
        !source.contains("foreground_color ="),
        "Workbench caption tone must resolve through the shared palette instead of a local RGB override"
    );
}

#[test]
fn blend_space_asset_declares_dense_adaptive_component_structure() {
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
        "component = \"WorkbenchSearchInput\"",
        "component = \"WorkbenchDivider\"",
        "component = \"WorkbenchCaption\"",
        "component = \"WorkbenchSampleGrid\"",
        "component = \"WorkbenchWeightHeatmap\"",
        "component = \"WorkbenchTimelineStrip\"",
        "component = \"WorkbenchSampleWeights\"",
        "component = \"WorkbenchValidationLog\"",
        "component = \"WorkbenchPreviewViewport\"",
        "component = \"WorkbenchBlendSpaceDetails\"",
        "control_id = \"WorkbenchExtensionBlendSpaceCenterWorkArea\"",
        "control_id = \"WorkbenchExtensionBlendSpaceTabs\"",
        "control_id = \"WorkbenchExtensionBlendSpaceSampleCanvas\"",
        "control_id = \"WorkbenchExtensionBlendSpaceSampleGrid\"",
        "control_id = \"WorkbenchExtensionBlendSpaceWeightHeatmap\"",
        "control_id = \"WorkbenchExtensionBlendSpacePreviewCard\"",
        "control_id = \"WorkbenchExtensionBlendSpaceOutputPanel\"",
        "control_id = \"WorkbenchExtensionBlendSpacePreviewTimeline\"",
        "control_id = \"WorkbenchExtensionBlendSpaceBottomCompositeRow\"",
        "responsive_min_tier = \"regular\"",
    ] {
        assert!(
            source.contains(required),
            "missing dense workspace contract: {required}"
        );
    }
    for required in [
        "component = \"WorkbenchPropertyRow\"",
        "component = \"WorkbenchTableRow\"",
        "component = \"WorkbenchSectionTitle\"",
    ] {
        assert!(
            details.contains(required),
            "missing extracted dense Details contract: {required}"
        );
    }
    assert!(source.contains("[nodes.blend_space_workspace]\ncomponent = \"HorizontalGroup\""));
    assert!(source.contains("[nodes.blend_space_left]\ncomponent = \"VerticalGroup\""));
    assert!(source.contains("[nodes.blend_space_right]\ncomponent = \"VerticalGroup\""));
    assert!(
        source.matches("stretch = \"Stretch\"").count() >= 24,
        "dense workspace should be governed primarily by relative stretch layout"
    );
    for bounded_width in [
        "width = { min = 188.0, preferred = 220.0, max = 250.0, stretch = \"Fixed\" }",
        "width = { min = 176.0, preferred = 208.0, max = 240.0, stretch = \"Fixed\" }",
        "width = { min = 210.0, preferred = 260.0, max = 310.0, stretch = \"Fixed\" }",
    ] {
        assert!(
            source.contains(bounded_width),
            "bounded side panes must use the shared Fixed axis contract: {bounded_width}"
        );
    }
    assert!(
        source
            .lines()
            .filter(|line| line.trim_start().starts_with("layout ="))
            .all(|line| !line.contains("weight =")),
        "Blend Space layouts must not introduce a private flex-weight dialect outside the shared schema"
    );
    assert!(source.contains(
        "[nodes.blend_space_center]\ncomponent = \"VerticalGroup\"\ncontrol_id = \"WorkbenchExtensionBlendSpaceCenterPanel\""
    ));
    assert!(source.contains(
        "[nodes.blend_space_sample_canvas]\ncomponent = \"VerticalGroup\"\ncontrol_id = \"WorkbenchExtensionBlendSpaceSampleCanvas\""
    ));
    assert!(source.contains(
        "[nodes.blend_space_sample_grid]\ncomponent = \"WorkbenchSampleGrid\"\ncontrol_id = \"WorkbenchExtensionBlendSpaceSampleGrid\""
    ));
    for typed_grid_contract in [
        "x_min = -180.0",
        "x_max = 180.0",
        "y_min = 0.0",
        "y_max = 600.0",
        "x_ticks = [-180.0, -135.0, -90.0, -45.0, 0.0, 45.0, 90.0, 135.0, 180.0]",
        "label = \"Run_Fwd\", selected = true",
    ] {
        assert!(
            source.contains(typed_grid_contract),
            "Blend Space must author typed sample-grid data: {typed_grid_contract}"
        );
    }
    for typed_heatmap_contract in [
        "heatmap_columns = 16",
        "heatmap_rows = 10",
        "heat_sources = [{ x = 0.5, y = 0.58, weight = 1.0, selected = true }",
    ] {
        assert!(
            source.contains(typed_heatmap_contract),
            "Blend Space must author typed weight-heatmap data: {typed_heatmap_contract}"
        );
    }
    for removed_placeholder in [
        "WorkbenchExtensionBlendSpaceForwardPoint",
        "WorkbenchExtensionBlendSpaceNeutralRow",
        "WorkbenchExtensionBlendSpaceBackwardPoint",
        "zircon_engine_style/scene/skeleton.svg",
        "control_id = \"WorkbenchExtensionBlendSpaceOutputLog\"",
        "control_id = \"WorkbenchExtensionBlendSpaceIdleSampleTableRow\"",
        "control_id = \"WorkbenchExtensionBlendSpaceWalkSampleTableRow\"",
        "control_id = \"WorkbenchExtensionBlendSpaceRunSampleTableRow\"",
        "control_id = \"WorkbenchExtensionBlendSpaceDiagonalSampleTableRow\"",
    ] {
        assert!(
            !source.contains(removed_placeholder),
            "sample grid must not fall back to list-row placeholder geometry: {removed_placeholder}"
        );
    }
    assert!(!source.contains("corner_radius = 6.0"));
    for forbidden in [
        "background_color =",
        "border_color =",
        "corner_radius =",
        "font_size =",
        "font_weight =",
        "text = \"/|\\\\\\n |\\n/ \\\\\"",
    ] {
        assert!(
            !source.contains(forbidden),
            "Blend Space composites must consume shared surface/image primitives instead of local visual overrides: {forbidden}"
        );
    }
}
