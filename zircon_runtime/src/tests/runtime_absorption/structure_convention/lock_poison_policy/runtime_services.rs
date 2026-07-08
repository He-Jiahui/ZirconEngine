use super::support::*;

#[path = "runtime_services/dynamic_scene.rs"]
mod dynamic_scene;
#[path = "runtime_services/navigation_resource.rs"]
mod navigation_resource;
#[path = "runtime_services/plugin_bridge.rs"]
mod plugin_bridge;

const TEST_ATTRIBUTE: &str = concat!("#[", "test", "]");

#[test]
fn runtime_15_runtime_services_lock_poison_guard_child_owner_split() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/lock_poison_policy/runtime_services.rs",
    );
    let plugin_bridge = read_runtime_src(
        "tests/runtime_absorption/structure_convention/lock_poison_policy/runtime_services/plugin_bridge.rs",
    );
    let dynamic_scene = read_runtime_src(
        "tests/runtime_absorption/structure_convention/lock_poison_policy/runtime_services/dynamic_scene.rs",
    );
    let navigation_resource = read_runtime_src(
        "tests/runtime_absorption/structure_convention/lock_poison_policy/runtime_services/navigation_resource.rs",
    );

    assert_contains_all(
        "runtime services lock-poison parent mounts child owners",
        &parent,
        &[
            "mod dynamic_scene;",
            "mod navigation_resource;",
            "mod plugin_bridge;",
        ],
    );

    for moved_guard in [
        concat!(
            "fn ",
            "runtime_15_plugin_bridge_table_lock_poison_recovery_guard_covers_provider_slot"
        ),
        concat!(
            "fn ",
            "runtime_15_navigation_lock_poison_recovery_guard_covers_builtin_navigation_manager"
        ),
        concat!(
            "fn ",
            "runtime_15_dynamic_api_session_lock_poison_recovery_guard_covers_session_registry"
        ),
        concat!(
            "fn ",
            "runtime_15_dynamic_scene_spawn_task_lock_poison_recovery_guard_covers_spawn_task"
        ),
        concat!(
            "fn ",
            "runtime_15_scene_ecs_parallel_executor_lock_poison_recovery_guard_covers_batch_result_slots"
        ),
        concat!(
            "fn ",
            "runtime_15_core_resource_manager_lock_poison_recovery_guard_covers_resource_manager"
        ),
    ] {
        assert!(
            !parent.contains(moved_guard),
            "runtime_services.rs should mount child owners instead of defining {moved_guard}"
        );
    }

    assert_contains_all(
        "plugin bridge child owns plugin lock-poison guard",
        &plugin_bridge,
        &[concat!(
            "fn ",
            "runtime_15_plugin_bridge_table_lock_poison_recovery_guard_covers_provider_slot"
        )],
    );
    assert_contains_all(
        "dynamic scene child owns dynamic API, spawn, and ECS lock-poison guards",
        &dynamic_scene,
        &[
            concat!(
                "fn ",
                "runtime_15_dynamic_api_session_lock_poison_recovery_guard_covers_session_registry"
            ),
            concat!(
                "fn ",
                "runtime_15_dynamic_scene_spawn_task_lock_poison_recovery_guard_covers_spawn_task"
            ),
            concat!(
                "fn ",
                "runtime_15_scene_ecs_parallel_executor_lock_poison_recovery_guard_covers_batch_result_slots"
            ),
        ],
    );
    assert_contains_all(
        "navigation/resource child owns navigation and resource lock-poison guards",
        &navigation_resource,
        &[
            concat!(
                "fn ",
                "runtime_15_navigation_lock_poison_recovery_guard_covers_builtin_navigation_manager"
            ),
            concat!(
                "fn ",
                "runtime_15_core_resource_manager_lock_poison_recovery_guard_covers_resource_manager"
            ),
        ],
    );

    let child_test_total = [
        parent.as_str(),
        plugin_bridge.as_str(),
        dynamic_scene.as_str(),
        navigation_resource.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches(TEST_ATTRIBUTE).count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 7,
        "runtime services lock-poison children should preserve six existing guards plus the new split guard"
    );

    for (path, source) in [
        (
            "structure_convention/lock_poison_policy/runtime_services.rs",
            parent.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/runtime_services/plugin_bridge.rs",
            plugin_bridge.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/runtime_services/dynamic_scene.rs",
            dynamic_scene.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/runtime_services/navigation_resource.rs",
            navigation_resource.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 400,
            "{path} should stay below the focused child-owner budget; got {line_count} lines"
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
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status.rs",
    );
    let status_map = [
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/lock_poison_module_maps.rs",
        ),
    ]
    .join("\n");
    let date_map = [
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/lock_poison_module_maps.rs",
        ),
    ]
    .join("\n");

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
        (
            "status-output M3 lock-poison row data",
            status_rows.as_str(),
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 runtime services lock-poison guard child-owner split",
                "runtime_15_runtime_services_lock_poison_guard_child_owner_split_static_passed_cargo_deferred",
                "structure_convention/lock_poison_policy/runtime_services.rs",
                "structure_convention/lock_poison_policy/runtime_services/plugin_bridge.rs",
                "structure_convention/lock_poison_policy/runtime_services/dynamic_scene.rs",
                "structure_convention/lock_poison_policy/runtime_services/navigation_resource.rs",
                "runtime_15_runtime_services_lock_poison_guard_child_owner_split",
            ],
        );
    }
    assert_contains_all(
        "status-output status map",
        &status_map,
        &[
            "Runtime 15 M3 runtime services lock-poison guard child-owner split",
            "runtime_15_runtime_services_lock_poison_guard_child_owner_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "status-output date map",
        &date_map,
        &[
            "Runtime 15 M3 runtime services lock-poison guard child-owner split",
            "2026-07-01",
        ],
    );
}
