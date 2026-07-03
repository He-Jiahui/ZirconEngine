pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 M2 row-data owner child split" {
        Some("runtime_15_m2_row_data_owner_child_split_static_passed_cargo_deferred")
        // Files: plan_status/status_output_tables/expected_status_row_data/runtime_15/m2.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m2/core_scene_asset_dynamic.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m2/render_graphics.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m2/ui_platform_editor.rs.
        // Guard: runtime_15_m2_row_data_owner_is_child_backed.
    } else if slice == "Runtime 15 M3 M2 row-data children guard folder-backed split" {
        Some("runtime_15_m2_row_data_children_guard_folder_backed_static_passed_cargo_deferred")
        // Files: structure_convention/test_file_budget/status_output_row_data/runtime_15_m2_row_data_children.rs; structure_convention/test_file_budget/status_output_row_data/runtime_15_m2_row_data_children/delegation.rs; structure_convention/test_file_budget/status_output_row_data/runtime_15_m2_row_data_children/row_ownership.rs; structure_convention/test_file_budget/status_output_row_data/runtime_15_m2_row_data_children/status_mirrors.rs; structure_convention/test_file_budget/status_output_row_data/runtime_15_m2_row_data_children/budgets.rs.
        // Guard: runtime_15_m2_row_data_children_guard_is_folder_backed.
    } else if slice == "Runtime 15 M1 animation manager folder-backed cutover" {
        // Evidence anchors: animation/manager/mod.rs and animation/manager/graph.rs.
        // Guard anchor: runtime_15_animation_manager_is_folder_backed.
        Some("runtime_15_animation_manager_folder_backed_cutover_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M2 core runtime state module naming hard cutover" {
        // Evidence anchor: core/runtime/state/core_runtime_state.rs.
        // Guard anchor: runtime_15_core_runtime_state_module_uses_owner_name.
        Some(
            "runtime_15_core_runtime_state_module_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice
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
    } else if slice == "Runtime 15 M2 plugin static manifest contract owner naming hard cutover" {
        // Evidence anchors:
        // plugin_extensions/static_manifest_contracts/feature_bundles/feature_bundle_rows.rs,
        // plugin_extensions/static_manifest_contracts/package_coordinates/package_coordinate_resolution.rs,
        // plugin_extensions/static_manifest_contracts/package_identity/package_id_tokens.rs,
        // plugin_extensions/static_manifest_contracts/package_kind/package_kind_fields.rs.
        // Guard anchor: runtime_15_plugin_static_manifest_contract_owners_use_domain_names.
        Some(
            "runtime_15_plugin_static_manifest_contract_owner_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice
        == "Runtime 15 M2 UI editor showcase descriptor builders module naming hard cutover"
    {
        // Evidence anchor: ui/component/catalog/editor_showcase/descriptor_builders.rs.
        // Guard anchor: runtime_15_ui_editor_showcase_descriptor_builders_use_owner_name.
        Some(
            "runtime_15_ui_editor_showcase_descriptor_builders_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M2 UI table sortingMode server literal allowed-context sync" {
        // Evidence anchor: ui/surface/surface/default_interactions/table/columns.rs.
        // Audit anchor: non_network_server_naming.py.
        // Guard anchors: runtime_non_network_server_naming_is_classified_by_owner,
        // runtime_15_ui_table_sorting_mode_server_literal_stays_allowed_context.
        Some(
            "runtime_15_ui_table_sorting_mode_server_literal_allowed_context_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M2 graphics render-framework receiver naming hard cutover" {
        // Evidence anchor: graphics/runtime/render_framework.
        // Receiver anchor: framework: &WgpuRenderFramework.
        // Guard anchors: runtime_non_network_server_naming_is_classified_by_owner,
        // runtime_15_render_framework_receiver_uses_framework_name.
        Some(
            "runtime_15_graphics_render_framework_receiver_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice
        == "Runtime 15 M2 render framework trait/construction owner naming hard cutover"
    {
        // Evidence anchors:
        // graphics/runtime/render_framework/render_framework_trait_binding/wgpu_framework.rs,
        // graphics/runtime/render_framework/wgpu_render_framework_construction/construct.rs.
        // Guard anchor: runtime_15_no_banned_name_modules.
        Some(
            "runtime_15_render_framework_trait_construction_owner_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M2 graphics construction new owner naming hard cutover" {
        // Evidence anchors:
        // graphics/feature/render_feature_descriptor/construct.rs,
        // graphics/scene/scene_renderer/post_process/resources/construct/construct/construct.rs.
        // Guard anchors: runtime_15_graphics_construction_new_owners_use_construct_names,
        // runtime_15_no_banned_name_modules.
        Some(
            "runtime_15_graphics_construction_new_owner_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M2 scene dynamic document v1 owner naming hard cutover" {
        // Evidence anchor: scene/dynamic_scene/document/v1_project_document.rs.
        // Type anchor: V1ProjectDocument.
        // Guard anchor: runtime_15_scene_dynamic_document_v1_owner_uses_versioned_name.
        Some(
            "runtime_15_scene_dynamic_document_v1_owner_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M2 scene render layer schema-v1 mask naming hard cutover" {
        // Evidence anchors:
        // core/framework/render/camera.rs and scene/world/render*.rs.
        // Helper anchor: from_scene_schema_v1_mask.
        // Guard anchor: runtime_15_scene_render_layer_schema_v1_masks_use_versioned_names.
        Some(
            "runtime_15_scene_render_layer_schema_v1_mask_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M2 render layer schema-v1 mask API naming hard cutover" {
        // Evidence anchors:
        // core/framework/render/camera.rs,
        // graphics/scene/scene_renderer/lighting/light_buffer.rs,
        // graphics/runtime/render_framework/viewport_record/camera_history_key.rs.
        // Helper anchors: from_scene_schema_v1_mask,
        // to_scene_schema_v1_mask_lossy, intersects_scene_schema_v1_mask.
        // Guard anchor: runtime_15_render_layer_schema_v1_mask_api_uses_current_names.
        Some(
            "runtime_15_render_layer_schema_v1_mask_api_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M2 render shader definition bare-flag naming hard cutover" {
        // Evidence anchor: core/framework/render/shader/definition_value.rs.
        // Serde branch anchor: BareFlag.
        // Guard anchor: runtime_15_render_shader_definition_uses_bare_flag_names.
        Some(
            "runtime_15_render_shader_definition_bare_flag_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M2 GPU model embedded primitive naming hard cutover" {
        // Evidence anchor:
        // graphics/scene/resources/gpu_model/gpu_model_resource_from_asset.rs.
        // Test-name anchor:
        // model_render_primitives_keep_embedded_payload_when_mesh_reference_unresolved.
        // Guard anchor: runtime_15_gpu_model_embedded_primitive_uses_current_names.
        Some(
            "runtime_15_gpu_model_embedded_primitive_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M2 frame extract snapshot adapter naming hard cutover" {
        // Evidence anchor: core/framework/render/frame_extract.rs.
        // Comment anchor: scene viewport snapshot packet.
        // Guard anchor: runtime_15_frame_extract_snapshot_adapter_uses_current_names.
        Some(
            "runtime_15_frame_extract_snapshot_adapter_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M2 core framework render fixture naming hard cutover" {
        // Evidence anchors:
        // core/framework/render/core_pipeline/render_queue.rs,
        // core/framework/render/post_process/effect_stack_settings.rs,
        // core/framework/render/relevance.rs,
        // core/framework/render/light/readiness.rs,
        // core/framework/render/scene_extract.rs.
        // Guard anchor: runtime_15_core_framework_render_fixtures_use_current_names.
        Some(
            "runtime_15_core_framework_render_fixture_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M2 render feature fallback capability naming hard cutover" {
        // Evidence anchors:
        // graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_enabled_features.rs,
        // graphics/scene/scene_renderer/core/runtime_features/runtime_features_from_pipeline.rs.
        // Fallback ID anchor: fallback-virtual-geometry-without-capability.
        // Guard anchor: runtime_15_render_feature_fallback_capability_fixtures_use_current_names.
        Some(
            "runtime_15_render_feature_fallback_capability_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M2 render material stale texture fixture naming hard cutover" {
        // Evidence anchor: graphics/scene/render_product_streamer_tests/material_runtime.rs.
        // Test-name anchor: unresolved_stale_texture.
        // URI anchor: res://textures/missing-stale-base.png.
        // Guard anchor: runtime_15_render_material_stale_texture_fixtures_use_current_names.
        Some(
            "runtime_15_render_material_stale_texture_fixture_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M2 render graph fallback fixture naming hard cutover" {
        // Evidence anchors:
        // graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_resources/scene_renderer_advanced_plugin_resources.rs,
        // graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/compute_workload.rs.
        // Unexpected dispatch anchor: unexpected-compute.
        // Guard anchor: runtime_15_render_graph_fallback_fixtures_use_current_names.
        Some(
            "runtime_15_render_graph_fallback_fixture_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M2 Hybrid GI extract scene-source naming hard cutover" {
        // Evidence anchor: zircon_plugins/hybrid_gi/runtime/src/hybrid_gi.
        // Helper anchors: extract_trace_region_ids, extract-backed,
        // extract-sourced RenderHybridGiProbe.
        // Guard anchor: runtime_15_hybrid_gi_extract_scene_source_uses_current_names.
        Some(
            "runtime_15_hybrid_gi_extract_scene_source_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M2 platform input DOM keycode naming hard cutover" {
        // Evidence anchor: ui/platform_input/keyboard_map.rs.
        // Function anchor: dom_key_code.
        // Guard anchor: runtime_15_platform_input_uses_dom_keycode_names.
        Some(
            "runtime_15_platform_input_dom_keycode_naming_hard_cutover_static_passed_cargo_timeout_no_result",
        )
    } else if slice
        == "Runtime 15 M2 platform input runtime baseline test naming hard cutover"
    {
        // Evidence anchor: ui/platform_input/winit_translation.rs.
        // Test-name anchor: runtime_input_baseline.
        // Guard anchor: runtime_15_platform_input_winit_tests_use_runtime_input_baseline_names.
        Some(
            "runtime_15_platform_input_runtime_baseline_test_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice
        == "Runtime 15 M2 UI template schema source fixture naming hard cutover"
    {
        // Evidence anchors:
        // ui/template/asset/schema/migrator.rs,
        // zircon_runtime_interface/src/ui/template/asset/schema/report.rs.
        // Enum anchor: SourceTemplateFixture.
        // Guard anchor: runtime_15_ui_template_schema_uses_source_fixture_names.
        Some(
            "runtime_15_ui_template_schema_source_fixture_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M2 input mouse-wheel line-delta naming hard cutover" {
        // Evidence anchors:
        // core/framework/input/mouse_wheel.rs,
        // input/runtime/default_input_manager.rs,
        // dynamic_api/session/events.rs.
        // Guard anchor: runtime_15_input_mouse_wheel_line_delta_uses_current_names.
        Some(
            "runtime_15_input_mouse_wheel_line_delta_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M2 DDS upload policy naming hard cutover" {
        // Evidence anchors:
        // asset/assets/texture/upload_support/dds.rs,
        // asset/tests/assets/texture_upload_readiness/container_fixtures.rs.
        // Helper anchors: dds_classic_fourcc_upload_layout, dds_classic_cubemap_bytes.
        // Guard anchor: runtime_15_dds_upload_policy_uses_classic_container_names.
        Some("runtime_15_dds_upload_policy_naming_hard_cutover_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M2 material asset schema-v1 defaults naming hard cutover" {
        // Evidence anchor: asset/assets/material/material_asset.rs.
        // Helper anchors: property_overrides_with_schema_v1_defaults,
        // texture_slots_with_schema_v1_defaults, schema_v1_pbr_texture_slots.
        // Guard owner: naming_boundary/runtime_15_m2/asset_schema.rs.
        // Guard anchor: runtime_15_material_asset_schema_v1_defaults_use_versioned_names.
        Some(
            "runtime_15_material_asset_schema_v1_defaults_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M2 font/UI asset schema naming hard cutover" {
        // Evidence anchors:
        // asset/assets/font.rs,
        // asset/importer/ingest/ui_v2_document_import.rs,
        // asset/importer/ingest/import_ui_zui_asset.rs.
        // Helper anchor: schema_v1_render_mode.
        // Guard anchor: runtime_15_font_ui_asset_schema_names_use_current_policy_terms.
        Some(
            "runtime_15_font_ui_asset_schema_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M2 font render-mode priority fixture naming hard cutover" {
        // Evidence anchor: graphics/scene/scene_renderer/ui/font_asset.rs.
        // Test-name anchor: schema_v1_render_mode_takes_priority_over_strategy_default_mode.
        // Guard anchor: runtime_15_font_render_mode_priority_fixture_uses_schema_v1_name.
        // Gate anchor: module_convention_gate classified-and-clear.
        Some(
            "runtime_15_font_render_mode_priority_fixture_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice
        == "Runtime 15 M2 Net HTTP backend Hyper HTTP/1 client policy hard cutover"
    {
        // Evidence anchors:
        // zircon_plugins/net/features/http/runtime/src/backend/client.rs,
        // zircon_plugins/net/features/http/runtime/src/backend/http1_client_policy.rs.
        // Audit anchor: external-hyper-http1-client-policy.
        // Guard anchor: runtime_15_net_http_hyper_http1_client_policy_is_isolated.
        Some(
            "runtime_15_net_http_hyper_http1_client_policy_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M2 Hub message raw text policy hard cutover" {
        // Evidence anchors:
        // zircon_hub/src/state/hub_message/message.rs,
        // zircon_hub/src/tauri_app/runtime_state/build_actions.rs.
        // Raw text anchor: HubMessage::raw_text.
        // Guard anchor: runtime_15_hub_message_raw_text_policy_uses_current_names.
        Some(
            "runtime_15_hub_message_raw_text_policy_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M2 editor workbench authority-label naming hard cutover" {
        // Evidence anchor:
        // zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/gameplay_state.rs.
        // Output anchor: Selected Condition_Night   editor authority.
        // Audit anchor: non_network_server_naming.py.
        // Guard anchor: runtime_15_editor_workbench_authority_label_uses_editor_name.
        Some(
            "runtime_15_editor_workbench_authority_label_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M2 editor Workbench archived fixture naming hard cutover" {
        // Evidence anchors:
        // zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/host_window.rs,
        // zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/cells/text.rs.
        // Entry anchor: draw_host_workbench_window.
        // Guard anchor: runtime_15_editor_workbench_archived_fixtures_use_current_names.
        Some(
            "runtime_15_editor_workbench_archived_fixture_naming_hard_cutover_static_passed_cargo_deferred",
        )
    } else {
        None
    }
}
