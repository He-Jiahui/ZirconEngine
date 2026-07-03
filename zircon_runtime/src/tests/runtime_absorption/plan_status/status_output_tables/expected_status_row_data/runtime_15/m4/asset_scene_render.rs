type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M4 material asset value/readiness helper owner split",
        &[
            "runtime_15_material_asset_value_readiness_owner_split_static_passed_cargo_timeout_no_result",
            "asset/assets/material/material_asset.rs",
            "asset/assets/material/material_asset/value_sync.rs",
            "runtime_15_material_asset_value_readiness_helpers_are_child_owners",
        ],
    ),
    (
        "Runtime 15 M4 material asset management record owner split",
        &[
            "runtime_15_material_asset_management_record_owner_split_static_passed_cargo_deferred",
            "asset/assets/material/material_asset.rs",
            "asset/assets/material/material_asset/management.rs",
            "runtime_15_material_asset_management_records_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 asset artifact cache UI document owner split",
        &[
            "runtime_15_asset_artifact_cache_ui_documents_owner_split_static_passed_cargo_deferred",
            "asset/artifact/cache_payload.rs",
            "asset/artifact/cache_payload/ui.rs",
            "runtime_15_asset_artifact_cache_ui_documents_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 asset artifact cache material/shader owner split",
        &[
            "runtime_15_asset_artifact_cache_material_shader_owner_split_static_passed_cargo_deferred",
            "asset/artifact/cache_payload.rs",
            "asset/artifact/cache_payload/material_shader.rs",
            "runtime_15_asset_artifact_cache_ui_documents_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 mesh asset management record owner split",
        &[
            "runtime_15_mesh_asset_management_record_owner_split_static_passed_cargo_deferred",
            "asset/assets/mesh/mesh_asset.rs",
            "asset/assets/mesh/mesh_asset/management.rs",
            "runtime_15_mesh_asset_management_records_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 asset project scan/import source collection owner split",
        &[
            "runtime_15_asset_project_scan_import_sources_owner_split_static_passed_cargo_deferred",
            "asset/project/manager/scan_and_import.rs",
            "asset/project/manager/scan_and_import/sources.rs",
            "runtime_15_asset_project_scan_import_sources_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 glTF labeled material subasset owner split",
        &[
            "runtime_15_gltf_labeled_material_subasset_owner_split_static_passed_cargo_deferred",
            "asset/importer/ingest/gltf_labeled_subassets.rs",
            "asset/importer/ingest/gltf_labeled_subassets/material.rs",
            "runtime_15_gltf_labeled_material_subassets_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 texture descriptor settings parser owner split",
        &[
            "runtime_15_texture_descriptor_settings_parser_owner_split_static_passed_cargo_deferred",
            "asset/assets/texture/descriptor.rs",
            "asset/assets/texture/descriptor/settings.rs",
            "runtime_15_texture_descriptor_settings_parser_is_child_owner",
        ],
    ),
    (
        "Runtime 15 F8 texture descriptor typed errors",
        &[
            "runtime_15_texture_descriptor_typed_errors_static_passed_cargo_deferred",
            "asset/assets/texture/descriptor.rs",
            "asset/assets/texture/descriptor/settings.rs",
            "asset/assets/texture/texture_asset.rs",
            "TextureDescriptorError",
            "review_f8_texture_import_settings_use_fallible_apply_not_with",
        ],
    ),
    (
        "Runtime 15 M4 scene world render light collection owner split",
        &[
            "runtime_15_scene_world_render_lights_owner_split_static_passed_cargo_deferred",
            "scene/world/render.rs",
            "scene/world/render/lights.rs",
            "runtime_15_scene_world_render_light_collectors_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 core runtime render-stats graph execution-resources owner split",
        &[
            "runtime_15_render_stats_graph_execution_resources_owner_split_static_passed_cargo_timeout_no_result",
            "core/runtime/diagnostics/render_stats_store/graph.rs",
            "core/runtime/diagnostics/render_stats_store/graph/execution_resources.rs",
            "runtime_15_render_stats_graph_execution_resources_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 render-stats product diagnostics test owner split",
        &[
            "runtime_15_render_stats_product_diagnostics_tests_owner_split_static_passed_cargo_deferred_active_editor_lane",
            "core/runtime/diagnostics/render_stats_store/product.rs",
            "core/runtime/diagnostics/render_stats_store/product/tests.rs",
            "core/runtime/diagnostics/render_stats_store/product/tests/mesh_gpu_scene.rs",
            "runtime_15_render_stats_product_diagnostics_tests_are_child_owners",
        ],
    ),
    (
        "Runtime 15 M4 extend pending draws material-input owner split",
        &[
            "runtime_15_extend_pending_draws_material_inputs_owner_split_static_passed_cargo_deferred",
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs",
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance/material_inputs.rs",
            "runtime_15_extend_pending_draws_tests_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 non-Base mesh variant render-call guard sync",
        &[
            "runtime_15_non_base_mesh_variant_render_call_guard_sync_static_passed_cargo_deferred",
            "graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs",
            "graphics/scene/scene_renderer/temporal/velocity/execute_velocity_object.rs",
            "runtime_15_non_base_mesh_variant_cache_owner_is_wired",
        ],
    ),
    (
        "Runtime 15 M4 shader prewarm manifest path helper owner split",
        &[
            "runtime_15_shader_prewarm_manifest_path_helpers_owner_split_static_passed_cargo_deferred",
            "bin/zircon_shader_prewarm/manifest.rs",
            "bin/zircon_shader_prewarm/manifest/paths.rs",
            "runtime_15_no_oversized_production_files",
            "runtime_15_shader_prewarm_manifest_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M4 scene fixed light reflection write-field owner split",
        &[
            "runtime_15_scene_fixed_light_reflection_write_fields_owner_split_static_passed_cargo_lock_blocked",
            "scene/reflect/fixed/lights.rs",
            "scene/reflect/fixed/lights/write_fields.rs",
            "runtime_15_scene_fixed_light_reflection_write_fields_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 scene world property-access physics write owner split",
        &[
            "runtime_15_scene_world_property_access_physics_owner_split_static_passed_cargo_timeout_no_result",
            "scene/world/property_access/write.rs",
            "scene/world/property_access/write/physics.rs",
            "runtime_15_scene_world_property_access_physics_writes_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 scene world property-access physics entry owner split",
        &[
            "runtime_15_scene_world_property_access_physics_entries_owner_split_static_passed_cargo_lock_blocked",
            "scene/world/property_access/entries.rs",
            "scene/world/property_access/entries/physics.rs",
            "runtime_15_scene_world_property_access_physics_entries_are_child_owner",
        ],
    ),
];
