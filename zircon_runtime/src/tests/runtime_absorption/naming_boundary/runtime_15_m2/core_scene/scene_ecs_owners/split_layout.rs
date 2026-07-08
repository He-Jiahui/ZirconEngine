use super::{
    assert_contains_all, CHILD_OWNER_GUARD, CHILD_OWNER_SLICE, CHILD_OWNER_STATUS, CHILD_PATHS,
    PARENT_PATH, SPLIT_LAYOUT_FRAMEWORKS_STATUS, SPLIT_LAYOUT_GUARD, SPLIT_LAYOUT_SLICE,
    SPLIT_LAYOUT_STATUS,
};

#[test]
fn runtime_15_core_scene_naming_ecs_owner_split_layout_is_folder_backed() {
    let route = include_str!("../scene_ecs_owners.rs");
    let observer = include_str!("observer_callback_registry.rs");
    let query_state = include_str!("query_state_many_item_array.rs");
    let component_storage = include_str!("component_storage_component_results.rs");
    let split_layout = include_str!("split_layout.rs");
    let children = format!("{observer}\n{query_state}\n{component_storage}\n{split_layout}");

    assert_contains_all(
        "core-scene scene-ECS owner route mounts child owners",
        route,
        &[
            "mod observer_callback_registry;",
            "mod query_state_many_item_array;",
            "mod component_storage_component_results;",
            "mod split_layout;",
        ],
    );
    assert_contains_all(
        "core-scene scene-ECS owner route keeps status constants",
        route,
        &[
            CHILD_OWNER_SLICE,
            CHILD_OWNER_STATUS,
            CHILD_OWNER_GUARD,
            SPLIT_LAYOUT_SLICE,
            SPLIT_LAYOUT_STATUS,
            SPLIT_LAYOUT_GUARD,
        ],
    );

    for moved_guard in [
        "runtime_15_scene_ecs_observer_callback_registry_uses_owner_name",
        "runtime_15_scene_ecs_query_state_many_item_array_uses_owner_name",
        "runtime_15_scene_ecs_component_storage_component_results_uses_owner_name",
    ] {
        assert!(
            !route.contains(&format!("fn {moved_guard}")),
            "scene_ecs_owners route should not retain `{moved_guard}`"
        );
        assert!(
            children.contains(&format!("fn {moved_guard}")),
            "scene_ecs_owners children should retain `{moved_guard}`"
        );
    }

    for (path, source, max_lines) in [
        (PARENT_PATH, route, 45usize),
        (CHILD_PATHS[0], observer, 160),
        (CHILD_PATHS[1], query_state, 170),
        (CHILD_PATHS[2], component_storage, 160),
        (CHILD_PATHS[3], split_layout, 190),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count <= max_lines,
            "{path} has {line_count} lines, expected <= {max_lines}"
        );
    }

    let status_rows = include_str!(
        "../../../../plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/asset_budget_tests.rs"
    );
    assert_contains_all(
        "Runtime 15 M3 asset-budget row data records scene-ECS split-layout",
        status_rows,
        &[
            SPLIT_LAYOUT_SLICE,
            SPLIT_LAYOUT_STATUS,
            SPLIT_LAYOUT_GUARD,
            PARENT_PATH,
            CHILD_PATHS[0],
            CHILD_PATHS[1],
            CHILD_PATHS[2],
            CHILD_PATHS[3],
        ],
    );

    let status_map = include_str!(
        "../../../../plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/naming_guard_maps.rs"
    );
    assert_contains_all(
        "Runtime 15 M3 naming status map records scene-ECS split-layout",
        status_map,
        &[SPLIT_LAYOUT_SLICE, SPLIT_LAYOUT_STATUS],
    );
    let date_map = include_str!(
        "../../../../plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/naming_guard_maps.rs"
    );
    assert_contains_all(
        "Runtime 15 M3 naming date map records scene-ECS split-layout",
        date_map,
        &[SPLIT_LAYOUT_SLICE, "2026-07-06"],
    );

    for (label, source) in [
        (
            "Runtime 15 subplan",
            include_str!(
                "../../../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
            ),
        ),
        (
            "runtime index",
            include_str!("../../../../../../../../docs/plans/zircon_runtime/runtime/index.md"),
        ),
        (
            "frameworks plan",
            include_str!(
                "../../../../../../../../docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md"
            ),
        ),
        (
            "engine code structure convention",
            include_str!("../../../../../../../../docs/plans/engine-code-structure-convention.md"),
        ),
        (
            "engine code review findings",
            include_str!(
                "../../../../../../../../docs/plans/engine-code-review-findings-2026-06.md"
            ),
        ),
        (
            "module convention doc",
            include_str!("../../../../../../../../docs/zircon_runtime/structure/module-convention.md"),
        ),
        (
            "runtime implementation session note",
            include_str!(
                "../../../../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
            ),
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                SPLIT_LAYOUT_SLICE,
                SPLIT_LAYOUT_STATUS,
                SPLIT_LAYOUT_GUARD,
                SPLIT_LAYOUT_FRAMEWORKS_STATUS,
                PARENT_PATH,
                CHILD_PATHS[0],
                CHILD_PATHS[1],
                CHILD_PATHS[2],
                CHILD_PATHS[3],
            ],
        );
    }
}
