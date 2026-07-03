use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_scene_world_render_light_collectors_are_child_owner() {
    let parent = read_runtime_src("scene/world/render.rs");
    let lights = read_runtime_src("scene/world/render/lights.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let render_extract_doc = read_repo("docs/zircon_runtime/scene/render_extract.md");
    let render_submit_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs",
    );
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m4_surface_cleanup.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m4_surface_cleanup.rs",
    );

    assert_contains_all(
        "scene world render parent keeps frame extract orchestration and shared layer helper",
        &parent,
        &[
            "mod lights;",
            "pub(crate) fn build_prepared_render_frame_extract_for_request(",
            "let ambient_lights = self.collect_ambient_lights(&camera_layers);",
            "let directional_lights = self.collect_directional_lights(&camera_layers);",
            "let point_lights = self.collect_point_lights(&camera_layers);",
            "let rect_lights = self.collect_rect_lights(&camera_layers);",
            "let spot_lights = self.collect_spot_lights(&camera_layers);",
            "pub(super) fn entity_intersects_camera_layers(",
        ],
    );
    for moved_owner in [
        "fn collect_ambient_lights(",
        "fn collect_directional_lights(",
        "fn collect_point_lights(",
        "fn collect_rect_lights(",
        "fn collect_spot_lights(",
        "Vec<RenderAmbientLightSnapshot>",
        "Vec<RenderDirectionalLightSnapshot>",
        "Vec<RenderPointLightSnapshot>",
        "Vec<RenderRectLightSnapshot>",
        "Vec<RenderSpotLightSnapshot>",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "scene/world/render.rs should delegate {moved_owner} to render/lights.rs"
        );
    }
    assert_contains_all(
        "scene world render light child owns light snapshot projection",
        &lights,
        &[
            "pub(super) fn collect_ambient_lights(",
            "pub(super) fn collect_directional_lights(",
            "pub(super) fn collect_point_lights(",
            "pub(super) fn collect_rect_lights(",
            "pub(super) fn collect_spot_lights(",
            "RenderAmbientLightSnapshot",
            "RenderDirectionalLightSnapshot",
            "RenderPointLightSnapshot",
            "RenderRectLightSnapshot",
            "RenderSpotLightSnapshot",
            "default_render_layer_mask()",
        ],
    );

    for (path, source) in [
        ("scene/world/render.rs", parent.as_str()),
        ("scene/world/render/lights.rs", lights.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("scene render-extract doc", render_extract_doc.as_str()),
        ("render product submit doc", render_submit_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M4 scene world render light collection owner split",
                "runtime_15_scene_world_render_lights_owner_split_static_passed_cargo_deferred",
                "scene/world/render.rs",
                "scene/world/render/lights.rs",
                "runtime_15_scene_world_render_light_collectors_are_child_owner",
            ],
        );
    }
    assert_contains_all(
        "status-output status map",
        &status_map,
        &[
            "Runtime 15 M4 scene world render light collection owner split",
            "runtime_15_scene_world_render_lights_owner_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "status-output date map",
        &date_map,
        &[
            "Runtime 15 M4 scene world render light collection owner split",
            "2026-06-24",
        ],
    );
}
