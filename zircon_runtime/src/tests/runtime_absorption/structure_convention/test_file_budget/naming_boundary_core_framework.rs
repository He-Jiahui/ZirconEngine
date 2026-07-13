use super::*;

const STATUS: &str = "runtime_15_core_framework_naming_camera_controller_guard_child_owner_split_static_passed_cargo_deferred";
const SLICE: &str = "Runtime 15 M3 core-framework naming camera-controller guard child-owner split";
const GUARD: &str = "runtime_15_core_framework_naming_camera_controller_guard_is_child_owner";

#[test]
fn runtime_15_core_framework_naming_camera_controller_guard_is_child_owner() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_framework.rs",
    );
    let child = read_runtime_src(
        "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_framework/camera_controller.rs",
    );

    assert_contains_all(
        "Runtime 15 core-framework naming parent mounts camera-controller child owner",
        &parent,
        &[
            "#[path = \"core_framework/camera_controller.rs\"]",
            "mod camera_controller;",
            "#[path = \"core_framework/render_fixtures.rs\"]",
            "mod render_fixtures;",
            "#[path = \"core_framework/render_layer_schema_v1.rs\"]",
            "mod render_layer_schema_v1;",
        ],
    );
    assert!(
        !parent.contains("fn runtime_15_camera_controller_output_uses_owner_name"),
        "runtime_15_m2/core_framework.rs should mount camera_controller child instead of defining the camera-controller naming guard"
    );
    assert!(
        !parent.contains("fn runtime_15_render_layer_schema_v1_mask_api_uses_current_names"),
        "runtime_15_m2/core_framework.rs should mount render_layer_schema_v1 child instead of defining the render-layer schema-v1 naming guard"
    );

    assert_contains_all(
        "Runtime 15 core-framework camera-controller child owns output naming guard",
        &child,
        &[
            "use super::*;",
            "fn runtime_15_camera_controller_output_uses_owner_name",
            "core/framework/camera_controller/controller_output.rs",
            "CursorGrabIntent",
            "CameraControllerOutput",
        ],
    );

    for (path, source) in [
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_framework.rs",
            parent.as_str(),
        ),
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_framework/camera_controller.rs",
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
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                SLICE,
                STATUS,
                GUARD,
                "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_framework.rs",
                "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_framework/camera_controller.rs",
                "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_framework/render_layer_schema_v1.rs",
            ],
        );
    }

    assert_contains_all(
        "Runtime 15 status/date maps record core-framework camera-controller child owner",
        &format!("{status_map}\n{date_map}"),
        &[SLICE, STATUS, "2026-06-30"],
    );
}
