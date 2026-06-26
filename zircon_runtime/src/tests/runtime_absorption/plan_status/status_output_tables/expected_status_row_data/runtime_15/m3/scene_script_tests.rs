use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 15 M3 Runtime 07 performance hotspot guard folder split",
        &[
            "runtime_15_runtime_07_performance_hotspots_guard_folder_split_static_passed_cargo_timeout_no_result",
            "tests/runtime_absorption/performance_hotspots.rs",
            "tests/runtime_absorption/performance_hotspots/submit_context.rs",
            "tests/runtime_absorption/performance_hotspots/hotspot_inventory.rs",
            "runtime_15_runtime_07_performance_hotspots_guard_is_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 script VM test folder split",
        &[
            "runtime_15_script_vm_tests_folder_split_static_passed_cargo_timeout_no_result",
            "script/vm/tests.rs",
            "script/vm/tests/host_exports.rs",
            "script/vm/tests/reflection_docs.rs",
            "runtime_15_script_vm_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 script VM hot-reload coordinator test folder split",
        &[
            "runtime_15_script_vm_hot_reload_coordinator_tests_folder_split_static_passed_cargo_deferred",
            "script/vm/runtime/hot_reload_coordinator.rs",
            "script/vm/runtime/hot_reload_coordinator/tests.rs",
            "runtime_15_script_vm_hot_reload_coordinator_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 native live-host tests folder split",
        &[
            "runtime_15_native_live_host_tests_folder_split_static_passed_cargo_deferred",
            "plugin/native_plugin_loader/native_plugin_live_host/tests.rs",
            "plugin/native_plugin_loader/native_plugin_live_host/tests/bridge_bindings.rs",
            "plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_state.rs",
            "runtime_15_native_live_host_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 native plugin loader real fixture test folder split",
        &[
            "runtime_15_native_plugin_loader_real_fixture_tests_folder_split_static_passed_cargo_deferred",
            "tests/plugin_extensions/native_plugin_loader.rs",
            "tests/plugin_extensions/native_plugin_loader/real_fixture.rs",
            "runtime_15_native_plugin_loader_real_fixture_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 extension registry bridge test folder split",
        &[
            "runtime_15_extension_registry_bridge_tests_folder_split_static_passed_cargo_deferred",
            "tests/plugin_extensions/extension_registry_bridge.rs",
            "tests/plugin_extensions/extension_registry_bridge/basics.rs",
            "tests/plugin_extensions/extension_registry_bridge/diagnostics.rs",
            "runtime_15_extension_registry_bridge_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 manifest contributions test folder split",
        &[
            "runtime_15_manifest_contributions_tests_folder_split_static_passed_cargo_deferred",
            "tests/plugin_extensions/manifest_contributions.rs",
            "tests/plugin_extensions/manifest_contributions/editor_only.rs",
            "tests/plugin_extensions/manifest_contributions/net.rs",
            "runtime_15_manifest_contributions_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 runtime plugin package manifest test folder split",
        &[
            "runtime_15_runtime_plugin_package_manifest_tests_folder_split_static_passed_cargo_deferred",
            "tests/plugin_extensions/runtime_plugin_package_manifest.rs",
            "tests/plugin_extensions/runtime_plugin_package_manifest/feature_modules.rs",
            "runtime_15_runtime_plugin_package_manifest_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 export build plan test folder split",
        &[
            "runtime_15_export_build_plan_tests_folder_split_static_passed_cargo_deferred",
            "tests/plugin_extensions/export_build_plan.rs",
            "tests/plugin_extensions/export_build_plan/catalog_projection.rs",
            "runtime_15_export_build_plan_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 export build plan platform test folder split",
        &[
            "runtime_15_export_build_plan_platform_tests_folder_split_static_passed_cargo_deferred",
            "tests/plugin_extensions/export_build_plan_platform.rs",
            "tests/plugin_extensions/export_build_plan_platform/browser_hosts.rs",
            "runtime_15_export_build_plan_platform_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 gameplay host test folder split",
        &[
            "runtime_15_gameplay_host_tests_folder_split_static_passed_cargo_deferred",
            "script/vm/gameplay_host/tests.rs",
            "script/vm/gameplay_host/tests/spawn_transform.rs",
            "script/vm/gameplay_host/tests/property_animation.rs",
            "runtime_15_gameplay_host_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 shader prewarm manifest test folder split",
        &[
            "runtime_15_shader_prewarm_manifest_tests_folder_split_static_passed_cargo_deferred",
            "bin/zircon_shader_prewarm/manifest.rs",
            "bin/zircon_shader_prewarm/manifest/tests.rs",
            "structure_convention/test_file_budget/shader_prewarm_manifest.rs",
            "runtime_15_shader_prewarm_manifest_tests_are_folder_backed",
        ],
    ),
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
    (
        "Runtime 15 M3 dynamic scene root test folder split",
        &[
            "runtime_15_dynamic_scene_root_tests_folder_split_static_passed_cargo_deferred",
            "scene/tests/dynamic_scene.rs",
            "scene/tests/dynamic_scene/archive_manifest.rs",
            "scene/tests/dynamic_scene/scene_patch_document.rs",
            "runtime_15_dynamic_scene_root_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 scene render extract test folder split",
        &[
            "runtime_15_scene_render_extract_tests_folder_split_static_passed_cargo_deferred",
            "scene/tests/render_extract.rs",
            "scene/tests/render_extract/direct_sections.rs",
            "scene/tests/render_extract/lighting_postprocess.rs",
            "runtime_15_scene_render_extract_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 scene asset integration test folder split",
        &[
            "runtime_15_scene_asset_integration_tests_folder_split_static_passed_cargo_deferred",
            "scene/tests/asset_scene.rs",
            "scene/tests/asset_scene/mesh_bindings.rs",
            "scene/tests/asset_scene/product_fields.rs",
            "runtime_15_scene_asset_integration_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 scene world basics test folder split",
        &[
            "runtime_15_scene_world_basics_tests_folder_split_static_passed_cargo_deferred",
            "scene/tests/world_basics.rs",
            "scene/tests/world_basics/render_extract.rs",
            "scene/tests/world_basics/sprites.rs",
            "runtime_15_scene_world_basics_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 scene property paths test folder split",
        &[
            "runtime_15_scene_property_paths_tests_folder_split_static_passed_cargo_deferred",
            "scene/tests/property_paths.rs",
            "scene/tests/property_paths/read_paths.rs",
            "scene/tests/property_paths/runtime_mutation.rs",
            "scene/tests/property_paths/write_validation.rs",
            "runtime_15_scene_property_paths_tests_are_folder_backed",
        ],
    ),
];
