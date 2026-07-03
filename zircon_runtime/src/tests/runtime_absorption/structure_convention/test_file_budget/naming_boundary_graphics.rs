use super::*;

const STATUS: &str =
    "runtime_15_graphics_naming_render_fixture_guard_child_owner_split_static_passed_cargo_deferred";
const SLICE: &str = "Runtime 15 M3 graphics naming render-fixture guard child-owner split";
const GUARD: &str = "runtime_15_graphics_naming_render_fixture_guards_are_child_owner";

#[test]
fn runtime_15_graphics_naming_render_fixture_guards_are_child_owner() {
    let parent =
        read_runtime_src("tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics.rs");
    let child = read_runtime_src(
        "tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics/render_fixtures.rs",
    );

    assert_contains_all(
        "Runtime 15 graphics naming parent mounts render-fixture child owner",
        &parent,
        &[
            "#[path = \"graphics/render_fixtures.rs\"]",
            "mod render_fixtures;",
            "#[path = \"graphics/render_framework_receiver.rs\"]",
            "mod render_framework_receiver;",
            "#[path = \"graphics/resource_streamer_construction.rs\"]",
            "mod resource_streamer_construction;",
            "#[path = \"graphics/offscreen_target_construct.rs\"]",
            "mod offscreen_target_construct;",
            "#[path = \"graphics/gpu_model_embedded_primitive.rs\"]",
            "mod gpu_model_embedded_primitive;",
            "fn rust_files(",
        ],
    );
    for moved_test in [
        "fn runtime_15_render_feature_fallback_capability_fixtures_use_current_names",
        "fn runtime_15_render_material_stale_texture_fixtures_use_current_names",
        "fn runtime_15_render_graph_fallback_fixtures_use_current_names",
        "fn runtime_15_render_framework_receiver_uses_framework_name",
        "fn runtime_15_resource_streamer_construction_uses_owner_name",
        "fn runtime_15_offscreen_target_construct_uses_owner_name",
        "fn runtime_15_gpu_model_embedded_primitive_uses_current_names",
    ] {
        assert!(
            !parent.contains(moved_test),
            "runtime_15_m2/graphics.rs should mount render_fixtures child instead of defining {moved_test}"
        );
    }

    assert_contains_all(
        "Runtime 15 graphics render-fixture child owns fixture naming guards",
        &child,
        &[
            "use super::*;",
            "fn runtime_15_render_feature_fallback_capability_fixtures_use_current_names",
            "fn runtime_15_render_material_stale_texture_fixtures_use_current_names",
            "fn runtime_15_render_graph_fallback_fixtures_use_current_names",
            "fallback-virtual-geometry-without-capability",
            "unresolved_stale_texture",
            "unexpected-compute",
        ],
    );

    for (path, source) in [
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics.rs",
            parent.as_str(),
        ),
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics/render_fixtures.rs",
            child.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/asset_budget_tests.rs",
    );
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/naming_guard_maps.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/naming_guard_maps.rs",
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                SLICE,
                STATUS,
                GUARD,
                "tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics.rs",
                "tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics/render_fixtures.rs",
            ],
        );
    }

    assert_contains_all(
        "Runtime 15 status/date maps record graphics naming render-fixture child owner",
        &format!("{status_map}\n{date_map}"),
        &[SLICE, STATUS, "2026-06-30"],
    );
}
