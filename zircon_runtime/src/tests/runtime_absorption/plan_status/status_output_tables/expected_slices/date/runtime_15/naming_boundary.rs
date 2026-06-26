pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 M1 animation manager folder-backed cutover" {
        // Status anchor:
        // runtime_15_animation_manager_folder_backed_cutover_static_passed_cargo_deferred.
        // Evidence anchors: animation/manager/mod.rs and animation/manager/graph.rs.
        // Guard anchor: runtime_15_animation_manager_is_folder_backed.
        Some("2026-06-24")
    } else if slice == "Runtime 15 M2 core runtime state module naming hard cutover" {
        // Status anchor:
        // runtime_15_core_runtime_state_module_naming_hard_cutover_static_passed_cargo_deferred.
        // Evidence anchor: core/runtime/state/core_runtime_state.rs.
        // Guard anchor: runtime_15_core_runtime_state_module_uses_owner_name.
        Some("2026-06-24")
    } else if slice
        == "Runtime 15 M2 scene ECS observer callback registry module naming hard cutover"
    {
        // Status anchor:
        // runtime_15_scene_ecs_observer_callback_registry_naming_hard_cutover_static_passed_cargo_deferred.
        // Evidence anchor: scene/ecs/observer/callback_registry.rs.
        // Guard anchor: runtime_15_scene_ecs_observer_callback_registry_uses_owner_name.
        Some("2026-06-24")
    } else if slice
        == "Runtime 15 M2 scene ECS query-state many-item array module naming hard cutover"
    {
        // Status anchor:
        // runtime_15_scene_ecs_query_state_many_item_array_naming_hard_cutover_static_passed_cargo_deferred.
        // Evidence anchor: scene/ecs/query/query_state/many_item_array.rs.
        // Guard anchor: runtime_15_scene_ecs_query_state_many_item_array_uses_owner_name.
        Some("2026-06-25")
    } else if slice
        == "Runtime 15 M2 scene ECS component-storage component results module naming hard cutover"
    {
        // Status anchor:
        // runtime_15_scene_ecs_component_storage_component_results_naming_hard_cutover_static_passed_cargo_deferred.
        // Evidence anchor: scene/ecs/storage/component_storage/component_results.rs.
        // Guard anchor: runtime_15_scene_ecs_component_storage_component_results_uses_owner_name.
        Some("2026-06-25")
    } else if slice == "Runtime 15 M2 asset watcher shutdown-on-drop module naming hard cutover" {
        // Status anchor:
        // runtime_15_asset_watcher_shutdown_on_drop_naming_hard_cutover_static_passed_cargo_deferred.
        // Evidence anchor: asset/watch/shutdown_on_drop.rs.
        // Guard anchor: runtime_15_asset_watcher_shutdown_on_drop_uses_owner_name.
        Some("2026-06-25")
    } else if slice == "Runtime 15 M2 asset change construction module naming hard cutover" {
        // Status anchor:
        // runtime_15_asset_change_construction_naming_hard_cutover_static_passed_cargo_deferred.
        // Evidence anchor: asset/watch/asset_change_construction.rs.
        // Guard anchor: runtime_15_asset_change_construction_uses_owner_name.
        Some("2026-06-25")
    } else if slice == "Runtime 15 M2 resource streamer construction module naming hard cutover" {
        // Status anchor:
        // runtime_15_resource_streamer_construction_naming_hard_cutover_static_passed_cargo_deferred.
        // Evidence anchor:
        // graphics/scene/resources/resource_streamer/resource_streamer_construction.rs.
        // Guard anchor: runtime_15_resource_streamer_construction_uses_owner_name.
        Some("2026-06-25")
    } else if slice == "Runtime 15 M2 offscreen target construct directory naming hard cutover" {
        // Status anchor:
        // runtime_15_offscreen_target_construct_naming_hard_cutover_static_passed_cargo_timeout_no_result.
        // Evidence anchor:
        // graphics/backend/render_backend/offscreen_target_construct/construct.rs.
        // Guard anchor: runtime_15_offscreen_target_construct_uses_owner_name.
        Some("2026-06-25")
    } else if slice
        == "Runtime 15 M2 asset texture upload readiness container fixtures module naming hard cutover"
    {
        // Status anchor:
        // runtime_15_asset_texture_upload_readiness_container_fixtures_naming_hard_cutover_static_passed_cargo_deferred.
        // Evidence anchor: asset/tests/assets/texture_upload_readiness/container_fixtures.rs.
        // Guard anchor:
        // runtime_15_asset_texture_upload_readiness_container_fixtures_uses_owner_name.
        Some("2026-06-25")
    } else if slice == "Runtime 15 M2 scene ECS query cached queries module naming hard cutover" {
        // Status anchor:
        // runtime_15_scene_ecs_query_cached_queries_naming_hard_cutover_static_passed_cargo_deferred.
        // Evidence anchor: scene/tests/ecs_query/cached_queries.rs.
        // Guard anchor: runtime_15_scene_ecs_query_cached_queries_uses_owner_name.
        Some("2026-06-25")
    } else if slice
        == "Runtime 15 M2 dynamic API vampire runtime support module naming hard cutover"
    {
        // Status anchor:
        // runtime_15_dynamic_api_vampire_runtime_support_naming_hard_cutover_static_passed_cargo_deferred.
        // Evidence anchor: dynamic_api/session/tests/vampire_runtime_support.rs.
        // Guard anchor: runtime_15_dynamic_api_vampire_runtime_support_uses_owner_name.
        Some("2026-06-25")
    } else if slice == "Runtime 15 M2 camera controller output module naming hard cutover" {
        // Status anchor:
        // runtime_15_camera_controller_output_naming_hard_cutover_static_passed_cargo_deferred.
        // Evidence anchor: core/framework/camera_controller/controller_output.rs.
        // Guard anchor: runtime_15_camera_controller_output_uses_owner_name.
        Some("2026-06-25")
    } else if slice
        == "Runtime 15 M2 scene ECS systems many/single queries module naming hard cutover"
    {
        // Status anchor:
        // runtime_15_scene_ecs_systems_many_single_queries_naming_hard_cutover_static_passed_cargo_deferred.
        // Evidence anchor: scene/tests/ecs_systems/many_single_queries.rs.
        // Guard anchor: runtime_15_scene_ecs_systems_many_single_queries_uses_owner_name.
        Some("2026-06-25")
    } else if slice == "Runtime 15 M2 plugin static manifest contract owner naming hard cutover" {
        // Status anchor:
        // runtime_15_plugin_static_manifest_contract_owner_naming_hard_cutover_static_passed_cargo_deferred.
        // Evidence anchors:
        // plugin_extensions/static_manifest_contracts/feature_bundles/feature_bundle_rows.rs,
        // plugin_extensions/static_manifest_contracts/package_coordinates/package_coordinate_resolution.rs,
        // plugin_extensions/static_manifest_contracts/package_identity/package_id_tokens.rs,
        // plugin_extensions/static_manifest_contracts/package_kind/package_kind_fields.rs.
        // Guard anchor: runtime_15_plugin_static_manifest_contract_owners_use_domain_names.
        Some("2026-06-25")
    } else if slice
        == "Runtime 15 M2 UI editor showcase descriptor builders module naming hard cutover"
    {
        // Status anchor:
        // runtime_15_ui_editor_showcase_descriptor_builders_naming_hard_cutover_static_passed_cargo_deferred.
        // Evidence anchor: ui/component/catalog/editor_showcase/descriptor_builders.rs.
        // Guard anchor: runtime_15_ui_editor_showcase_descriptor_builders_use_owner_name.
        Some("2026-06-25")
    } else if slice == "Runtime 15 M2 UI table sortingMode server literal allowed-context sync" {
        // Status anchor:
        // runtime_15_ui_table_sorting_mode_server_literal_allowed_context_static_passed_cargo_deferred.
        // Evidence anchor: ui/surface/surface/default_interactions/table/columns.rs.
        // Audit anchor: non_network_server_naming.py.
        // Guard anchors: runtime_non_network_server_naming_is_classified_by_owner,
        // runtime_15_ui_table_sorting_mode_server_literal_stays_allowed_context.
        Some("2026-06-25")
    } else if slice == "Runtime 15 M2 graphics render-framework receiver naming hard cutover" {
        // Status anchor:
        // runtime_15_graphics_render_framework_receiver_naming_hard_cutover_static_passed_cargo_deferred.
        // Evidence anchor: graphics/runtime/render_framework.
        // Receiver anchor: framework: &WgpuRenderFramework.
        // Guard anchors: runtime_non_network_server_naming_is_classified_by_owner,
        // runtime_15_render_framework_receiver_uses_framework_name.
        Some("2026-06-25")
    } else if slice == "Runtime 15 M2 editor workbench authority-label naming hard cutover" {
        // Status anchor:
        // runtime_15_editor_workbench_authority_label_naming_hard_cutover_static_passed_cargo_deferred.
        // Evidence anchor:
        // zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/gameplay_state.rs.
        // Output anchor: Selected Condition_Night   editor authority.
        // Audit anchor: non_network_server_naming.py.
        // Guard anchor: runtime_15_editor_workbench_authority_label_uses_editor_name.
        Some("2026-06-25")
    } else {
        None
    }
}
