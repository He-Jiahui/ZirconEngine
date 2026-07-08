pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if slice
        == "Runtime 15 M2 scene ECS observer callback registry module naming hard cutover"
    {
        // Evidence anchor: scene/ecs/observer/callback_registry.rs.
        // Guard anchor: runtime_15_scene_ecs_observer_callback_registry_uses_owner_name.
        Some(
            "runtime_15_scene_ecs_observer_callback_registry_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice
        == "Runtime 15 M2 scene ECS query-state many-item array module naming hard cutover"
    {
        // Evidence anchor: scene/ecs/query/query_state/many_item_array.rs.
        // Guard anchor: runtime_15_scene_ecs_query_state_many_item_array_uses_owner_name.
        Some(
            "runtime_15_scene_ecs_query_state_many_item_array_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice
        == "Runtime 15 M2 scene ECS component-storage component results module naming hard cutover"
    {
        // Evidence anchor: scene/ecs/storage/component_storage/component_results.rs.
        // Guard anchor: runtime_15_scene_ecs_component_storage_component_results_uses_owner_name.
        Some(
            "runtime_15_scene_ecs_component_storage_component_results_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M2 asset watcher shutdown-on-drop module naming hard cutover" {
        // Evidence anchor: asset/watch/shutdown_on_drop.rs.
        // Guard anchor: runtime_15_asset_watcher_shutdown_on_drop_uses_owner_name.
        Some(
            "runtime_15_asset_watcher_shutdown_on_drop_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M2 asset change construction module naming hard cutover" {
        // Evidence anchor: asset/watch/asset_change_construction.rs.
        // Guard anchor: runtime_15_asset_change_construction_uses_owner_name.
        Some(
            "runtime_15_asset_change_construction_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M2 resource streamer construction module naming hard cutover" {
        // Evidence anchor:
        // graphics/scene/resources/resource_streamer/resource_streamer_construction.rs.
        // Guard anchor: runtime_15_resource_streamer_construction_uses_owner_name.
        Some(
            "runtime_15_resource_streamer_construction_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M2 offscreen target construct directory naming hard cutover" {
        // Evidence anchor:
        // graphics/backend/render_backend/offscreen_target_construct/construct.rs.
        // Guard anchor: runtime_15_offscreen_target_construct_uses_owner_name.
        Some(
            "runtime_15_offscreen_target_construct_naming_hard_cutover_static_passed_cargo_timeout_no_result",
        )
    } else if slice
        == "Runtime 15 M2 asset texture upload readiness container fixtures module naming hard cutover"
    {
        // Evidence anchor: asset/tests/assets/texture_upload_readiness/container_fixtures.rs.
        // Guard anchor: runtime_15_asset_texture_upload_readiness_container_fixtures_uses_owner_name.
        Some(
            "runtime_15_asset_texture_upload_readiness_container_fixtures_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M2 scene ECS query cached queries module naming hard cutover" {
        // Evidence anchor: scene/tests/ecs_query/cached_queries.rs.
        // Guard anchor: runtime_15_scene_ecs_query_cached_queries_uses_owner_name.
        Some(
            "runtime_15_scene_ecs_query_cached_queries_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice
        == "Runtime 15 M2 dynamic API vampire runtime support module naming hard cutover"
    {
        // Evidence anchor: dynamic_api/session/tests/vampire_runtime_support.rs.
        // Guard anchor: runtime_15_dynamic_api_vampire_runtime_support_uses_owner_name.
        Some(
            "runtime_15_dynamic_api_vampire_runtime_support_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M2 camera controller output module naming hard cutover" {
        // Evidence anchor: core/framework/camera_controller/controller_output.rs.
        // Guard anchor: runtime_15_camera_controller_output_uses_owner_name.
        Some("runtime_15_camera_controller_output_naming_hard_cutover_static_passed_cargo_deferred")
    } else if slice
        == "Runtime 15 M2 scene ECS systems many/single queries module naming hard cutover"
    {
        // Evidence anchor: scene/tests/ecs_systems/many_single_queries.rs.
        // Guard anchor: runtime_15_scene_ecs_systems_many_single_queries_uses_owner_name.
        Some(
            "runtime_15_scene_ecs_systems_many_single_queries_naming_hard_cutover_static_passed_cargo_timeout_no_result",
        )
    } else {
        None
    }
}
