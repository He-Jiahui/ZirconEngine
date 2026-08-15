use super::support::*;
use super::*;

#[test]
fn blend_space_preview_timeline_uses_shared_typed_canvas_and_preserves_actions() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = std::fs::read_to_string(manifest.join(
        "assets/ui/editor/components/workbench/modules/extensions/animation/\
             workbench_extension_blend_space_workspace.zui",
    ))
    .expect("Blend Space workspace asset should be readable");
    let primitive = std::fs::read_to_string(manifest.join(
        "assets/ui/editor/components/workbench/primitives/data/workbench_timeline_strip.zui",
    ))
    .expect("timeline strip primitive should be readable");
    let details = std::fs::read_to_string(manifest.join(
        "assets/ui/editor/components/workbench/composites/animation/\
             workbench_blend_space_details.zui",
    ))
    .expect("shared Blend Space details composite should be readable");

    for required in [
        "[components.WorkbenchTimelineStrip]",
        "component = \"Canvas\"",
        "component_variant = \"timeline-strip\"",
        "duration = 3.0",
        "current_time = 3.0",
        "tick_interval = 0.5",
        "track_label = \"Run_Fwd\"",
        "time = 2.0, label = \"Run_Fwd\", selected = true",
    ] {
        assert!(
            primitive.contains(required) || source.contains(required),
            "missing typed preview-timeline contract: {required}"
        );
    }
    for preserved_action in [
        "route = \"workbench.extension.blend_space.samples_tab.select\"",
        "route = \"workbench.extension.blend_space.axes_tab.select\"",
        "route = \"workbench.extension.blend_space.preview_tab.select\"",
        "route = \"workbench.extension.blend_space.idle_run_row.select\"",
        "route = \"workbench.extension.blend_space.strafe_row.select\"",
        "route = \"workbench.extension.blend_space.sprint_row.select\"",
        "route = \"workbench.extension.blend_space.output.select\"",
        "route = \"workbench.extension.blend_space.preview.invoke\"",
        "route = \"workbench.extension.blend_space.apply.invoke\"",
        "route = \"workbench.extension.blend_space.idle_sample_table_row.select\"",
        "route = \"workbench.extension.blend_space.walk_sample_table_row.select\"",
        "route = \"workbench.extension.blend_space.run_sample_table_row.select\"",
        "route = \"workbench.extension.blend_space.diagonal_sample_table_row.select\"",
    ] {
        assert!(
            source.contains(preserved_action) || details.contains(preserved_action),
            "timeline/sample refactor must preserve existing interaction route: {preserved_action}"
        );
    }
    for deprecated_route in [
        "route = \"workbench.extension.blend_space.samples\"",
        "route = \"workbench.extension.blend_space.axes\"",
        "route = \"workbench.extension.blend_space.preview_tab\"",
        "route = \"workbench.extension.blend_space.asset.idle_run\"",
        "route = \"workbench.extension.blend_space.asset.strafe\"",
        "route = \"workbench.extension.blend_space.asset.sprint\"",
        "route = \"workbench.extension.blend_space.output\"",
        "route = \"workbench.extension.blend_space.preview\"",
        "route = \"workbench.extension.blend_space.apply\"",
    ] {
        assert!(
            !source.contains(deprecated_route),
            "workspace must use its canonical binding action, not {deprecated_route}"
        );
    }
}
