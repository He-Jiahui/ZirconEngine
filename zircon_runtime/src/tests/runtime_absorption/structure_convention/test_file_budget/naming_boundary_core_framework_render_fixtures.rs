use super::*;

const STATUS: &str = "runtime_15_core_framework_naming_render_fixture_guard_child_owner_split_static_passed_cargo_deferred";
const SLICE: &str = "Runtime 15 M3 core-framework naming render-fixture guard child-owner split";
const GUARD: &str = "runtime_15_core_framework_naming_render_fixture_guard_is_child_owner";

#[test]
fn runtime_15_core_framework_naming_render_fixture_guard_is_child_owner() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_framework.rs",
    );
    let child = read_runtime_src(
        "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_framework/render_fixtures.rs",
    );

    assert_contains_all(
        "Runtime 15 core-framework naming parent mounts render-fixture child owner",
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
        !parent.contains("fn runtime_15_core_framework_render_fixtures_use_current_names"),
        "runtime_15_m2/core_framework.rs should mount render_fixtures child instead of defining the render-fixture naming guard"
    );
    assert!(
        !parent.contains("fn runtime_15_render_layer_schema_v1_mask_api_uses_current_names"),
        "runtime_15_m2/core_framework.rs should mount render_layer_schema_v1 child instead of defining the render-layer schema-v1 naming guard"
    );

    assert_contains_all(
        "Runtime 15 core-framework render-fixture child owns render naming guard",
        &child,
        &[
            "use super::*;",
            "fn runtime_15_core_framework_render_fixtures_use_current_names",
            "core/framework/render/core_pipeline/render_queue.rs",
            "authored_queue_offsets_are_clamped_to_material_window",
            "scene_schema_v1_mask",
        ],
    );

    for (path, source) in [
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_framework.rs",
            parent.as_str(),
        ),
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_framework/render_fixtures.rs",
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
}
