use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
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
    (
        "Runtime 15 M2 UI editor showcase descriptor builders module naming hard cutover",
        &[
            "runtime_15_ui_editor_showcase_descriptor_builders_naming_hard_cutover_static_passed_cargo_deferred",
            "ui/component/catalog/editor_showcase/descriptor_builders.rs",
            "ui/component/catalog/editor_showcase.rs",
            "runtime_15_ui_editor_showcase_descriptor_builders_use_owner_name",
        ],
    ),
    (
        "Runtime 15 M2 UI table sortingMode server literal allowed-context sync",
        &[
            "runtime_15_ui_table_sorting_mode_server_literal_allowed_context_static_passed_cargo_deferred",
            "ui/surface/surface/default_interactions/table/columns.rs",
            "non_network_server_naming.py",
            "runtime_non_network_server_naming_is_classified_by_owner",
            "runtime_15_ui_table_sorting_mode_server_literal_stays_allowed_context",
        ],
    ),
    (
        "Runtime 15 M2 graphics render-framework receiver naming hard cutover",
        &[
            "runtime_15_graphics_render_framework_receiver_naming_hard_cutover_static_passed_cargo_deferred",
            "graphics/runtime/render_framework",
            "framework: &WgpuRenderFramework",
            "runtime_non_network_server_naming_is_classified_by_owner",
            "runtime_15_render_framework_receiver_uses_framework_name",
        ],
    ),
    (
        "Runtime 15 M2 render framework trait/construction owner naming hard cutover",
        &[
            "runtime_15_render_framework_trait_construction_owner_naming_hard_cutover_static_passed_cargo_deferred",
            "graphics/runtime/render_framework/render_framework_trait_binding/wgpu_framework.rs",
            "graphics/runtime/render_framework/wgpu_render_framework_construction/construct.rs",
            "runtime_15_no_banned_name_modules",
        ],
    ),
    (
        "Runtime 15 M2 graphics construction new owner naming hard cutover",
        &[
            "runtime_15_graphics_construction_new_owner_naming_hard_cutover_static_passed_cargo_deferred",
            "graphics/feature/render_feature_descriptor/construct.rs",
            "graphics/scene/scene_renderer/post_process/resources/construct/construct/construct.rs",
            "runtime_15_graphics_construction_new_owners_use_construct_names",
            "runtime_15_no_banned_name_modules",
        ],
    ),
    (
        "Runtime 15 M2 scene dynamic document v1 owner naming hard cutover",
        &[
            "runtime_15_scene_dynamic_document_v1_owner_naming_hard_cutover_static_passed_cargo_deferred",
            "scene/dynamic_scene/document/v1_project_document.rs",
            "V1ProjectDocument",
            "runtime_15_scene_dynamic_document_v1_owner_uses_versioned_name",
        ],
    ),
    (
        "Runtime 15 M2 scene render layer schema-v1 mask naming hard cutover",
        &[
            "runtime_15_scene_render_layer_schema_v1_mask_naming_hard_cutover_static_passed_cargo_deferred",
            "core/framework/render/camera.rs",
            "scene/world/render.rs",
            "scene/world/render_particles.rs",
            "from_scene_schema_v1_mask",
            "runtime_15_scene_render_layer_schema_v1_masks_use_versioned_names",
        ],
    ),
    (
        "Runtime 15 M2 render layer schema-v1 mask API naming hard cutover",
        &[
            "runtime_15_render_layer_schema_v1_mask_api_naming_hard_cutover_static_passed_cargo_deferred",
            "core/framework/render/camera.rs",
            "graphics/scene/scene_renderer/lighting/light_buffer.rs",
            "graphics/runtime/render_framework/viewport_record/camera_history_key.rs",
            "from_scene_schema_v1_mask",
            "to_scene_schema_v1_mask_lossy",
            "intersects_scene_schema_v1_mask",
            "runtime_15_render_layer_schema_v1_mask_api_uses_current_names",
        ],
    ),
    (
        "Runtime 15 M2 render shader definition bare-flag naming hard cutover",
        &[
            "runtime_15_render_shader_definition_bare_flag_naming_hard_cutover_static_passed_cargo_deferred",
            "core/framework/render/shader/definition_value.rs",
            "BareFlag",
            "runtime_15_render_shader_definition_uses_bare_flag_names",
        ],
    ),
    (
        "Runtime 15 M2 GPU model embedded primitive naming hard cutover",
        &[
            "runtime_15_gpu_model_embedded_primitive_naming_hard_cutover_static_passed_cargo_deferred",
            "graphics/scene/resources/gpu_model/gpu_model_resource_from_asset.rs",
            "embedded primitive",
            "model_render_primitives_keep_embedded_payload_when_mesh_reference_unresolved",
            "runtime_15_gpu_model_embedded_primitive_uses_current_names",
        ],
    ),
    (
        "Runtime 15 M2 frame extract snapshot adapter naming hard cutover",
        &[
            "runtime_15_frame_extract_snapshot_adapter_naming_hard_cutover_static_passed_cargo_deferred",
            "core/framework/render/frame_extract.rs",
            "scene viewport snapshot packet",
            "RenderFrameExtract::from_snapshot",
            "runtime_15_frame_extract_snapshot_adapter_uses_current_names",
        ],
    ),
    (
        "Runtime 15 M2 core framework render fixture naming hard cutover",
        &[
            "runtime_15_core_framework_render_fixture_naming_hard_cutover_static_passed_cargo_deferred",
            "core/framework/render/core_pipeline/render_queue.rs",
            "scene_schema_v1_mask",
            "extended_effect_stack_settings_enable_product_node_without_retired_fields",
            "runtime_15_core_framework_render_fixtures_use_current_names",
        ],
    ),
    (
        "Runtime 15 M2 render feature fallback capability naming hard cutover",
        &[
            "runtime_15_render_feature_fallback_capability_naming_hard_cutover_static_passed_cargo_deferred",
            "graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_enabled_features.rs",
            "graphics/scene/scene_renderer/core/runtime_features/runtime_features_from_pipeline.rs",
            "fallback-virtual-geometry-without-capability",
            "runtime_15_render_feature_fallback_capability_fixtures_use_current_names",
        ],
    ),
    (
        "Runtime 15 M2 render material stale texture fixture naming hard cutover",
        &[
            "runtime_15_render_material_stale_texture_fixture_naming_hard_cutover_static_passed_cargo_deferred",
            "graphics/scene/render_product_streamer_tests/material_runtime.rs",
            "unresolved_stale_texture",
            "res://textures/missing-stale-base.png",
            "runtime_15_render_material_stale_texture_fixtures_use_current_names",
        ],
    ),
    (
        "Runtime 15 M2 render graph fallback fixture naming hard cutover",
        &[
            "runtime_15_render_graph_fallback_fixture_naming_hard_cutover_static_passed_cargo_deferred",
            "graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_resources/scene_renderer_advanced_plugin_resources.rs",
            "graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/compute_workload.rs",
            "unexpected-compute",
            "runtime_15_render_graph_fallback_fixtures_use_current_names",
        ],
    ),
    (
        "Runtime 15 M2 Hybrid GI extract scene-source naming hard cutover",
        &[
            "runtime_15_hybrid_gi_extract_scene_source_naming_hard_cutover_static_passed_cargo_deferred",
            "zircon_plugins/hybrid_gi/runtime/src/hybrid_gi",
            "extract_trace_region_ids",
            "extract-backed",
            "extract-sourced RenderHybridGiProbe",
            "runtime_15_hybrid_gi_extract_scene_source_uses_current_names",
        ],
    ),
    (
        "Runtime 15 M2 platform input DOM keycode naming hard cutover",
        &[
            "runtime_15_platform_input_dom_keycode_naming_hard_cutover_static_passed_cargo_timeout_no_result",
            "ui/platform_input/keyboard_map.rs",
            "dom_key_code",
            "runtime_15_platform_input_uses_dom_keycode_names",
        ],
    ),
    (
        "Runtime 15 M2 platform input runtime baseline test naming hard cutover",
        &[
            "runtime_15_platform_input_runtime_baseline_test_naming_hard_cutover_static_passed_cargo_deferred",
            "ui/platform_input/winit_translation.rs",
            "runtime_input_baseline",
            "runtime_15_platform_input_winit_tests_use_runtime_input_baseline_names",
        ],
    ),
    (
        "Runtime 15 M2 UI template schema source fixture naming hard cutover",
        &[
            "runtime_15_ui_template_schema_source_fixture_naming_hard_cutover_static_passed_cargo_deferred",
            "ui/template/asset/schema/migrator.rs",
            "SourceTemplateFixture",
            "runtime_15_ui_template_schema_uses_source_fixture_names",
        ],
    ),
    (
        "Runtime 15 M2 input mouse-wheel line-delta naming hard cutover",
        &[
            "runtime_15_input_mouse_wheel_line_delta_naming_hard_cutover_static_passed_cargo_deferred",
            "core/framework/input/mouse_wheel.rs",
            "vertical_line_delta",
            "runtime_15_input_mouse_wheel_line_delta_uses_current_names",
        ],
    ),
    (
        "Runtime 15 M2 DDS upload policy naming hard cutover",
        &[
            "runtime_15_dds_upload_policy_naming_hard_cutover_static_passed_cargo_deferred",
            "asset/assets/texture/upload_support/dds.rs",
            "asset/tests/assets/texture_upload_readiness/container_fixtures.rs",
            "dds_classic_fourcc_upload_layout",
            "dds_classic_cubemap_bytes",
            "runtime_15_dds_upload_policy_uses_classic_container_names",
        ],
    ),
    (
        "Runtime 15 M2 material asset schema-v1 defaults naming hard cutover",
        &[
            "runtime_15_material_asset_schema_v1_defaults_naming_hard_cutover_static_passed_cargo_deferred",
            "asset/assets/material/material_asset.rs",
            "property_overrides_with_schema_v1_defaults",
            "texture_slots_with_schema_v1_defaults",
            "schema_v1_pbr_texture_slots",
            "naming_boundary/runtime_15_m2/asset_schema.rs",
            "runtime_15_material_asset_schema_v1_defaults_use_versioned_names",
        ],
    ),
    (
        "Runtime 15 M2 Net HTTP backend Hyper HTTP/1 client policy hard cutover",
        &[
            "runtime_15_net_http_hyper_http1_client_policy_hard_cutover_static_passed_cargo_deferred",
            "zircon_plugins/net/features/http/runtime/src/backend/client.rs",
            "zircon_plugins/net/features/http/runtime/src/backend/http1_client_policy.rs",
            "http1_client_policy::plain_http_client()",
            "external-hyper-http1-client-policy",
            "runtime_15_net_http_hyper_http1_client_policy_is_isolated",
        ],
    ),
    (
        "Runtime 15 M2 Hub message raw text policy hard cutover",
        &[
            "runtime_15_hub_message_raw_text_policy_hard_cutover_static_passed_cargo_deferred",
            "zircon_hub/src/state/hub_message/message.rs",
            "zircon_hub/src/tauri_app/runtime_state/build_actions.rs",
            "HubMessage::raw_text",
            "runtime_15_hub_message_raw_text_policy_uses_current_names",
        ],
    ),
    (
        "Runtime 15 M2 editor workbench authority-label naming hard cutover",
        &[
            "runtime_15_editor_workbench_authority_label_naming_hard_cutover_static_passed_cargo_deferred",
            "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/gameplay_state.rs",
            "Selected Condition_Night   editor authority",
            "non_network_server_naming.py",
            "runtime_15_editor_workbench_authority_label_uses_editor_name",
        ],
    ),
    (
        "Runtime 15 M2 editor Workbench archived fixture naming hard cutover",
        &[
            "runtime_15_editor_workbench_archived_fixture_naming_hard_cutover_static_passed_cargo_deferred",
            "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/host_window.rs",
            "draw_host_workbench_window",
            "split_archived_table_text",
            "WorkbenchExtensionIconLibraryArchivedTableRow",
            "runtime_15_editor_workbench_archived_fixtures_use_current_names",
        ],
    ),
];
