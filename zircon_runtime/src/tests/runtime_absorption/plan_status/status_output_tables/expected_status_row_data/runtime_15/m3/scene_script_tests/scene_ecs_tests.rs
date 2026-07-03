type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 scene ECS schedule test folder split",
        &[
            "runtime_15_scene_ecs_schedule_tests_folder_split_static_passed_cargo_deferred",
            "scene/tests/ecs_schedule.rs",
            "scene/tests/ecs_schedule/resources_events.rs",
            "scene/tests/ecs_schedule/render_extract.rs",
            "runtime_15_scene_ecs_schedule_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 scene ECS schedule conflict graph child folder split",
        &[
            "runtime_15_scene_ecs_schedule_conflict_graph_child_folder_split_static_passed_cargo_deferred",
            "scene/tests/ecs_schedule/conflict_graph.rs",
            "scene/tests/ecs_schedule/conflict_graph/access_conflicts.rs",
            "scene/tests/ecs_schedule/conflict_graph/parallel_batches.rs",
            "runtime_15_scene_ecs_schedule_conflict_graph_children_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 scene ECS systems test folder split",
        &[
            "runtime_15_scene_ecs_systems_tests_folder_split_static_passed_cargo_deferred",
            "scene/tests/ecs_systems.rs",
            "scene/tests/ecs_systems/run_window_filters.rs",
            "scene/tests/ecs_systems/state_params.rs",
            "runtime_15_scene_ecs_systems_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 scene ECS query test folder split",
        &[
            "runtime_15_scene_ecs_query_tests_folder_split_static_passed_cargo_deferred",
            "scene/tests/ecs_query.rs",
            "scene/tests/ecs_query/cached_queries.rs",
            "scene/tests/ecs_query/mutation_access.rs",
            "runtime_15_scene_ecs_query_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 scene ECS query structure test folder split",
        &[
            "runtime_15_scene_ecs_query_structure_tests_folder_split_static_passed_cargo_deferred",
            "scene/tests/ecs_query_structure.rs",
            "scene/tests/ecs_query_structure/cache_rebuild.rs",
            "scene/tests/ecs_query_structure/cached_iterators.rs",
            "runtime_15_scene_ecs_query_structure_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 scene derived-state test folder split",
        &[
            "runtime_15_scene_derived_state_tests_folder_split_static_passed_cargo_deferred",
            "scene/tests/derived_state.rs",
            "scene/tests/derived_state/projected_reads.rs",
            "scene/tests/derived_state/runtime_freshness.rs",
            "runtime_15_scene_derived_state_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 dynamic scene session path-management test folder split",
        &[
            "runtime_15_dynamic_scene_session_path_management_tests_folder_split_static_passed_cargo_deferred",
            "scene/tests/dynamic_scene_session/path_management.rs",
            "scene/tests/dynamic_scene_session/path_management/single_slot_import.rs",
            "scene/tests/dynamic_scene_session/path_management/archive_merge.rs",
            "runtime_15_dynamic_scene_session_path_management_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 scene component-structure test folder split",
        &[
            "runtime_15_scene_component_structure_tests_folder_split_static_passed_cargo_deferred",
            "scene/tests/component_structure.rs",
            "scene/tests/component_structure/component_storage_indexing.rs",
            "scene/tests/component_structure/dynamic_scene_owner_tree.rs",
            "runtime_15_scene_component_structure_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 scene ECS reflect foundation test folder split",
        &[
            "runtime_15_scene_ecs_reflect_foundation_tests_folder_split_static_passed_cargo_deferred",
            "scene/tests/ecs_reflect/foundation.rs",
            "scene/tests/ecs_reflect/foundation/value_conversion.rs",
            "scene/tests/ecs_reflect/foundation/fixed_render_physics.rs",
            "runtime_15_scene_ecs_reflect_foundation_tests_are_folder_backed",
        ],
    ),
];
