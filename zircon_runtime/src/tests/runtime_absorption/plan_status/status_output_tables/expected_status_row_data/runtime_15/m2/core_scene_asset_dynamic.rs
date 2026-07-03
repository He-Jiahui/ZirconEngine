type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 status output Runtime 15 M2 row data split",
        &[
            "runtime_15_status_output_runtime_15_m2_row_data_split_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m2.rs",
            "runtime_15_status_output_runtime_15_m2_row_data_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M2 core runtime state module naming hard cutover",
        &[
            "runtime_15_core_runtime_state_module_naming_hard_cutover_static_passed_cargo_deferred",
            "core/runtime/state/core_runtime_state.rs",
            "core/runtime/state/mod.rs",
            "runtime_15_core_runtime_state_module_uses_owner_name",
        ],
    ),
    (
        "Runtime 15 M2 scene ECS observer callback registry module naming hard cutover",
        &[
            "runtime_15_scene_ecs_observer_callback_registry_naming_hard_cutover_static_passed_cargo_deferred",
            "scene/ecs/observer/callback_registry.rs",
            "scene/ecs/observer/mod.rs",
            "runtime_15_scene_ecs_observer_callback_registry_uses_owner_name",
        ],
    ),
    (
        "Runtime 15 M2 scene ECS query-state many-item array module naming hard cutover",
        &[
            "runtime_15_scene_ecs_query_state_many_item_array_naming_hard_cutover_static_passed_cargo_deferred",
            "scene/ecs/query/query_state/many_item_array.rs",
            "scene/ecs/query/query_state/mod.rs",
            "runtime_15_scene_ecs_query_state_many_item_array_uses_owner_name",
        ],
    ),
    (
        "Runtime 15 M2 scene ECS component-storage component results module naming hard cutover",
        &[
            "runtime_15_scene_ecs_component_storage_component_results_naming_hard_cutover_static_passed_cargo_deferred",
            "scene/ecs/storage/component_storage/component_results.rs",
            "scene/ecs/storage/component_storage/mod.rs",
            "runtime_15_scene_ecs_component_storage_component_results_uses_owner_name",
        ],
    ),
    (
        "Runtime 15 M2 asset watcher shutdown-on-drop module naming hard cutover",
        &[
            "runtime_15_asset_watcher_shutdown_on_drop_naming_hard_cutover_static_passed_cargo_deferred",
            "asset/watch/shutdown_on_drop.rs",
            "asset/watch/mod.rs",
            "runtime_15_asset_watcher_shutdown_on_drop_uses_owner_name",
        ],
    ),
    (
        "Runtime 15 M2 asset change construction module naming hard cutover",
        &[
            "runtime_15_asset_change_construction_naming_hard_cutover_static_passed_cargo_deferred",
            "asset/watch/asset_change_construction.rs",
            "asset/watch/mod.rs",
            "runtime_15_asset_change_construction_uses_owner_name",
        ],
    ),
    (
        "Runtime 15 M2 resource streamer construction module naming hard cutover",
        &[
            "runtime_15_resource_streamer_construction_naming_hard_cutover_static_passed_cargo_deferred",
            "graphics/scene/resources/resource_streamer/resource_streamer_construction.rs",
            "graphics/scene/resources/resource_streamer/mod.rs",
            "runtime_15_resource_streamer_construction_uses_owner_name",
        ],
    ),
    (
        "Runtime 15 M2 offscreen target construct directory naming hard cutover",
        &[
            "runtime_15_offscreen_target_construct_naming_hard_cutover_static_passed_cargo_timeout_no_result",
            "graphics/backend/render_backend/offscreen_target_construct/construct.rs",
            "graphics/backend/render_backend/mod.rs",
            "runtime_15_offscreen_target_construct_uses_owner_name",
        ],
    ),
    (
        "Runtime 15 M2 asset texture upload readiness container fixtures module naming hard cutover",
        &[
            "runtime_15_asset_texture_upload_readiness_container_fixtures_naming_hard_cutover_static_passed_cargo_deferred",
            "asset/tests/assets/texture_upload_readiness/container_fixtures.rs",
            "asset/tests/assets/texture_upload_readiness.rs",
            "runtime_15_asset_texture_upload_readiness_container_fixtures_uses_owner_name",
        ],
    ),
    (
        "Runtime 15 M2 scene ECS query cached queries module naming hard cutover",
        &[
            "runtime_15_scene_ecs_query_cached_queries_naming_hard_cutover_static_passed_cargo_deferred",
            "scene/tests/ecs_query/cached_queries.rs",
            "scene/tests/ecs_query.rs",
            "runtime_15_scene_ecs_query_cached_queries_uses_owner_name",
        ],
    ),
    (
        "Runtime 15 M2 dynamic API vampire runtime support module naming hard cutover",
        &[
            "runtime_15_dynamic_api_vampire_runtime_support_naming_hard_cutover_static_passed_cargo_deferred",
            "dynamic_api/session/tests/vampire_runtime_support.rs",
            "dynamic_api/session/tests/mod.rs",
            "runtime_15_dynamic_api_vampire_runtime_support_uses_owner_name",
        ],
    ),
    (
        "Runtime 15 M2 camera controller output module naming hard cutover",
        &[
            "runtime_15_camera_controller_output_naming_hard_cutover_static_passed_cargo_deferred",
            "core/framework/camera_controller/controller_output.rs",
            "core/framework/camera_controller/mod.rs",
            "runtime_15_camera_controller_output_uses_owner_name",
        ],
    ),
    (
        "Runtime 15 M2 scene ECS systems many/single queries module naming hard cutover",
        &[
            "runtime_15_scene_ecs_systems_many_single_queries_naming_hard_cutover_static_passed_cargo_timeout_no_result",
            "scene/tests/ecs_systems/many_single_queries.rs",
            "scene/tests/ecs_systems.rs",
            "runtime_15_scene_ecs_systems_many_single_queries_uses_owner_name",
        ],
    ),
    (
        "Runtime 15 M2 plugin static manifest contract owner naming hard cutover",
        &[
            "runtime_15_plugin_static_manifest_contract_owner_naming_hard_cutover_static_passed_cargo_deferred",
            "plugin_extensions/static_manifest_contracts/feature_bundles/feature_bundle_rows.rs",
            "plugin_extensions/static_manifest_contracts/package_coordinates/package_coordinate_resolution.rs",
            "plugin_extensions/static_manifest_contracts/package_identity/package_id_tokens.rs",
            "plugin_extensions/static_manifest_contracts/package_kind/package_kind_fields.rs",
            "runtime_15_plugin_static_manifest_contract_owners_use_domain_names",
        ],
    ),
];
