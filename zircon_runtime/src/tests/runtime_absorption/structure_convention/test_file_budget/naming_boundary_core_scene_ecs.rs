use super::*;

const STATUS: &str =
    "runtime_15_core_scene_naming_ecs_owner_guard_child_owner_split_static_passed_cargo_deferred";
const SLICE: &str = "Runtime 15 M3 core-scene naming ECS owner guard child-owner split";
const GUARD: &str = "runtime_15_core_scene_naming_ecs_owner_guards_are_child_owner";

#[test]
fn runtime_15_core_scene_naming_ecs_owner_guards_are_child_owner() {
    let parent =
        read_runtime_src("tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene.rs");
    let child = read_runtime_src(
        "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners.rs",
    );
    let observer_callback_registry = read_runtime_src(
        "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners/observer_callback_registry.rs",
    );
    let query_state_many_item_array = read_runtime_src(
        "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners/query_state_many_item_array.rs",
    );
    let component_storage_component_results = read_runtime_src(
        "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners/component_storage_component_results.rs",
    );
    let split_layout = read_runtime_src(
        "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners/split_layout.rs",
    );

    assert_contains_all(
        "Runtime 15 core-scene naming parent mounts scene-ECS child owner",
        &parent,
        &[
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
        "fn runtime_15_scene_ecs_observer_callback_registry_uses_owner_name",
        "fn runtime_15_scene_ecs_query_state_many_item_array_uses_owner_name",
        "fn runtime_15_scene_ecs_component_storage_component_results_uses_owner_name",
    ] {
        assert!(
            !parent.contains(moved_test),
            "runtime_15_m2/core_scene.rs should mount scene_ecs_owners child instead of defining {moved_test}"
        );
    }

    assert_contains_all(
        "Runtime 15 core-scene scene-ECS child mounts split ECS naming guards",
        &child,
        &[
            "use super::*;",
            "mod component_storage_component_results;",
            "mod observer_callback_registry;",
            "mod query_state_many_item_array;",
            "mod split_layout;",
        ],
    );

    assert_contains_all(
        "Runtime 15 core-scene scene-ECS child files own ECS naming guards",
        &format!(
            "{observer_callback_registry}\n{query_state_many_item_array}\n{component_storage_component_results}\n{split_layout}"
        ),
        &[
            "use super::*;",
            "fn runtime_15_scene_ecs_observer_callback_registry_uses_owner_name",
            "fn runtime_15_scene_ecs_query_state_many_item_array_uses_owner_name",
            "fn runtime_15_scene_ecs_component_storage_component_results_uses_owner_name",
            "scene/ecs/observer/callback_registry.rs",
            "scene/ecs/query/query_state/many_item_array.rs",
            "scene/ecs/storage/component_storage/component_results.rs",
        ],
    );

    for (path, source) in [
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene.rs",
            parent.as_str(),
        ),
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners.rs",
            child.as_str(),
        ),
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners/observer_callback_registry.rs",
            observer_callback_registry.as_str(),
        ),
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners/query_state_many_item_array.rs",
            query_state_many_item_array.as_str(),
        ),
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners/component_storage_component_results.rs",
            component_storage_component_results.as_str(),
        ),
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners/split_layout.rs",
            split_layout.as_str(),
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
                "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene.rs",
                "tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners.rs",
            ],
        );
    }

    assert_contains_all(
        "Runtime 15 status/date maps record core-scene scene-ECS child owner",
        &format!("{status_map}\n{date_map}"),
        &[SLICE, STATUS, "2026-06-30"],
    );
}
