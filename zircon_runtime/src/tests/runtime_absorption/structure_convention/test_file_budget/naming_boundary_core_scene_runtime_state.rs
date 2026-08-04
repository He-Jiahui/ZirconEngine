use super::*;

const STATUS: &str =
    "runtime_15_core_scene_runtime_state_guard_child_owner_split_static_passed_cargo_deferred";
const SLICE: &str = "Runtime 15 M3 core-scene runtime-state guard child-owner split";
const GUARD: &str = "runtime_15_core_scene_runtime_state_guard_is_child_owner";

#[test]
fn runtime_15_core_scene_runtime_state_guard_is_child_owner() {
    let parent =
        read_runtime_src("tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene.rs");
    let child = read_runtime_src(
        "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/core_runtime_state.rs",
    );

    assert_contains_all(
        "Runtime 15 core-scene naming parent mounts runtime-state child owner",
        &parent,
        &[
            "#[path = \"core_scene/core_runtime_state.rs\"]",
            "mod core_runtime_state;",
            "#[path = \"core_scene/render_contracts.rs\"]",
            "mod render_contracts;",
            "#[path = \"core_scene/scene_ecs_owners.rs\"]",
            "mod scene_ecs_owners;",
            "#[path = \"core_scene/render_layer_schema_v1.rs\"]",
            "mod render_layer_schema_v1;",
        ],
    );
    for moved_test in [
        "fn runtime_15_core_runtime_state_module_uses_owner_name",
        "fn runtime_15_scene_render_layer_schema_v1_masks_use_versioned_names",
    ] {
        assert!(
            !parent.contains(moved_test),
            "runtime_15_m2/core_scene.rs should mount child owners instead of defining {moved_test}"
        );
    }

    assert_contains_all(
        "Runtime 15 core-scene runtime-state child owns guard",
        &child,
        &[
            "use super::*;",
            "fn runtime_15_core_runtime_state_module_uses_owner_name",
            "core/runtime/state/core_runtime_state.rs",
            "runtime_15_core_runtime_state_module_naming_hard_cutover_static_passed_cargo_deferred",
        ],
    );

    for (path, source) in [
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene.rs",
            parent.as_str(),
        ),
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/core_runtime_state.rs",
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
