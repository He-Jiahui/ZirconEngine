use super::ExpectedStatusOutputSlice;

#[path = "runtime_15/foundation.rs"]
mod foundation;
#[path = "runtime_15/m2.rs"]
mod m2;
#[path = "runtime_15/m3.rs"]
mod m3;
#[path = "runtime_15/m4.rs"]
mod m4;

pub(super) const RUNTIME_15_FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = foundation::FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_FOUNDATION_TYPED_ERROR_RUNTIME_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    foundation::FOUNDATION_TYPED_ERROR_RUNTIME_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_FOUNDATION_TYPED_ERROR_PLUGIN_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    foundation::FOUNDATION_TYPED_ERROR_PLUGIN_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_FOUNDATION_TYPED_ERROR_SCENE_ASSET_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    foundation::FOUNDATION_TYPED_ERROR_SCENE_ASSET_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M2_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    m2::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M2_RENDER_GRAPHICS_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = m2::RENDER_GRAPHICS_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M2_UI_PLATFORM_EDITOR_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = m2::UI_PLATFORM_EDITOR_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M2_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = m2::ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M4_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    m4::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M4_ASSET_SCENE_RENDER_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = m4::ASSET_SCENE_RENDER_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M4_UI_TEXT_TEMPLATE_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = m4::UI_TEXT_TEMPLATE_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M4_UI_SURFACE_PLUGIN_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = m4::UI_SURFACE_PLUGIN_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M4_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = m4::ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M4_RENDER_SHADER_SYNC_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = m4::RENDER_SHADER_SYNC_EXPECTED_STATUS_OUTPUT_SLICES;

pub(super) const RUNTIME_15_F12_RESOURCE_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 15 F12 offscreen target texture owner cleanup",
        &[
            "runtime_15_offscreen_target_texture_owner_cleanup_static_passed_cargo_timeout_no_result",
            "graphics/backend/render_backend/offscreen_target.rs",
            "docs/zircon_runtime/graphics/render-product-submit.md",
            "runtime_15_offscreen_target_texture_owner_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 render backend state owner cleanup",
        &[
            "runtime_15_render_backend_state_owner_cleanup_coremin_check_passed",
            "graphics/backend/render_backend/render_backend.rs",
            "docs/zircon_runtime/graphics/render-product-submit.md",
            "runtime_15_render_backend_state_owner_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 gpu texture resource owner cleanup",
        &[
            "runtime_15_gpu_texture_resource_owner_cleanup_coremin_check_passed",
            "graphics/scene/resources/gpu_texture/gpu_texture_resource.rs",
            "docs/zircon_runtime/graphics/render-product-submit.md",
            "runtime_15_gpu_texture_resource_owner_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 gpu material uniform owner cleanup",
        &[
            "runtime_15_gpu_material_uniform_owner_cleanup_coremin_check_passed",
            "graphics/scene/resources/gpu_material_uniform/gpu_material_uniform_resource.rs",
            "docs/zircon_runtime/graphics/render-product-submit.md",
            "runtime_15_gpu_material_uniform_owner_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 gpu mesh order signature cleanup",
        &[
            "runtime_15_gpu_mesh_order_signature_cleanup_coremin_check_passed",
            "graphics/scene/resources/gpu_mesh/gpu_mesh_resource.rs",
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs",
            "runtime_15_gpu_mesh_order_signature_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 gpu model identity cleanup",
        &[
            "runtime_15_gpu_model_identity_cleanup_coremin_check_passed",
            "graphics/scene/resources/gpu_model/gpu_model_resource.rs",
            "graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs",
            "runtime_15_gpu_model_identity_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 post-process LUT texture owner cleanup",
        &[
            "runtime_15_post_process_lut_texture_owner_cleanup_coremin_check_passed",
            "graphics/scene/resources/post_process_lut_texture/post_process_lut_texture_resource.rs",
            "graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs",
            "runtime_15_post_process_lut_texture_owner_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 output target texture owner cleanup",
        &[
            "runtime_15_output_target_texture_owner_cleanup_coremin_check_passed",
            "graphics/scene/resources/output_target_texture/output_target_texture_resource.rs",
            "graphics/scene/resources/prepared/prepared_output_target_texture.rs",
            "runtime_15_output_target_texture_owner_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 material runtime capture seed cleanup",
        &[
            "runtime_15_material_runtime_capture_seed_cleanup_coremin_check_passed",
            "graphics/scene/resources/runtime/material_runtime.rs",
            "graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs",
            "runtime_15_material_runtime_capture_seed_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 resource streamer diagnostics accessor cleanup",
        &[
            "runtime_15_resource_streamer_diagnostics_accessor_cleanup_static_passed_cargo_lock_blocked",
            "graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs",
            "resource_streamer_ensure_scene_resources.rs",
            "runtime_15_resource_streamer_diagnostics_accessor_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 ResourceStreamer material diagnostics child owner split",
        &[
            "runtime_15_resource_streamer_material_diagnostics_child_owner_split_static_passed_cargo_deferred",
            "graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs",
            "graphics/scene/resources/resource_streamer/resource_streamer_accessors/material_diagnostics.rs",
            "runtime_15_resource_streamer_diagnostics_accessor_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 resource streamer resolve texture id cleanup",
        &[
            "runtime_15_resource_streamer_resolve_texture_id_cleanup_static_passed_cargo_lock_blocked",
            "graphics/scene/resources/resource_streamer/resource_streamer_resolve_texture_id.rs",
            "resolve_texture_reference_with_support",
            "runtime_15_resource_streamer_resolve_texture_id_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 particle GPU readback output accessor cleanup",
        &[
            "runtime_15_particle_gpu_readback_output_accessor_cleanup_static_passed_cargo_lock_blocked",
            "graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/take_last_particle_gpu_readback_outputs.rs",
            "renderer.take_last_particle_gpu_readback_outputs()",
            "runtime_15_particle_gpu_readback_output_accessor_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 advanced plugin output test accessor cleanup",
        &[
            "runtime_15_advanced_plugin_output_test_accessor_cleanup_static_passed_cargo_lock_blocked",
            "graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_access.rs",
            "has_particle_gpu_readback",
            "runtime_15_advanced_plugin_output_test_accessor_cleanup",
        ],
    ),
];

pub(super) const RUNTIME_15_M3_FOUNDATION_GUARD_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = m3::FOUNDATION_GUARD_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_FOUNDATION_GUARD_RUNTIME_STRUCTURE_TESTS_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::FOUNDATION_GUARD_RUNTIME_STRUCTURE_TESTS_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_FOUNDATION_GUARD_PLUGIN_IMPORTER_REVIEW_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::FOUNDATION_GUARD_PLUGIN_IMPORTER_REVIEW_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_FOUNDATION_GUARD_PLUGIN_IMPORTER_MIGRATIONS_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::FOUNDATION_GUARD_PLUGIN_IMPORTER_MIGRATIONS_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_FOUNDATION_GUARD_RUNTIME_ABSORPTION_FOLLOWUPS_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::FOUNDATION_GUARD_RUNTIME_ABSORPTION_FOLLOWUPS_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_FOUNDATION_GUARD_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::FOUNDATION_GUARD_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_LOCK_POISON_STATUS_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = m3::LOCK_POISON_STATUS_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_LOCK_POISON_STATUS_POLICY_GUARDS_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::LOCK_POISON_STATUS_POLICY_GUARDS_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_LOCK_POISON_STATUS_CORE_RUNTIME_RECOVERY_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::LOCK_POISON_STATUS_CORE_RUNTIME_RECOVERY_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_LOCK_POISON_STATUS_RUNTIME_SERVICES_RECOVERY_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::LOCK_POISON_STATUS_RUNTIME_SERVICES_RECOVERY_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_LOCK_POISON_STATUS_RESOURCE_RENDER_INPUT_RECOVERY_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::LOCK_POISON_STATUS_RESOURCE_RENDER_INPUT_RECOVERY_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_LOCK_POISON_STATUS_SCRIPT_VM_RECOVERY_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::LOCK_POISON_STATUS_SCRIPT_VM_RECOVERY_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_LOCK_POISON_STATUS_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::LOCK_POISON_STATUS_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_MODULE_CONVENTION_STATUS_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = m3::MODULE_CONVENTION_STATUS_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_REVIEW_STATUS_SYNC_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = m3::REVIEW_STATUS_SYNC_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_REVIEW_GUARD_CODE_REVIEW_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = m3::REVIEW_GUARD_CODE_REVIEW_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_REVIEW_GUARD_CODE_REVIEW_DIRECT_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::REVIEW_GUARD_CODE_REVIEW_DIRECT_ASSERTION_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_REVIEW_GUARD_CODE_REVIEW_PLUGIN_IMPORTER_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::REVIEW_GUARD_CODE_REVIEW_PLUGIN_IMPORTER_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_REVIEW_GUARD_CODE_REVIEW_STRUCTURE_GUARD_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::REVIEW_GUARD_CODE_REVIEW_STRUCTURE_GUARD_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_REVIEW_GUARD_CODE_REVIEW_STRUCTURE_GUARD_STATUS_DOCS_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::REVIEW_GUARD_CODE_REVIEW_STRUCTURE_GUARD_STATUS_DOCS_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_REVIEW_GUARD_CODE_REVIEW_STRUCTURE_GUARD_FOLDER_BACKED_SUMMARY_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::REVIEW_GUARD_CODE_REVIEW_STRUCTURE_GUARD_FOLDER_BACKED_SUMMARY_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_REVIEW_GUARD_CODE_REVIEW_STRUCTURE_GUARD_TYPED_ERROR_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::REVIEW_GUARD_CODE_REVIEW_STRUCTURE_GUARD_TYPED_ERROR_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_REVIEW_GUARD_CODE_REVIEW_STRUCTURE_GUARD_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::REVIEW_GUARD_CODE_REVIEW_STRUCTURE_GUARD_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_REVIEW_GUARD_CODE_REVIEW_TYPED_ERROR_STRUCTURE_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::REVIEW_GUARD_CODE_REVIEW_TYPED_ERROR_STRUCTURE_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_REVIEW_GUARD_CODE_REVIEW_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::REVIEW_GUARD_CODE_REVIEW_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_REVIEW_GUARD_STATUS_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = m3::REVIEW_GUARD_STATUS_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_REVIEW_GUARD_TYPED_ERROR_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = m3::REVIEW_GUARD_TYPED_ERROR_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_REVIEW_GUARD_TYPED_ERROR_RUNTIME_SURFACE_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::REVIEW_GUARD_TYPED_ERROR_RUNTIME_SURFACE_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_REVIEW_GUARD_TYPED_ERROR_ASSET_SHADER_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::REVIEW_GUARD_TYPED_ERROR_ASSET_SHADER_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_UI_TESTS_FIRST_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = m3::UI_TESTS_FIRST_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_ASSET_BUDGET_TESTS_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = m3::ASSET_BUDGET_TESTS_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_ASSET_BUDGET_ASSET_TESTS_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = m3::ASSET_BUDGET_ASSET_TESTS_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_ASSET_BUDGET_BUDGET_RENDER_UI_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = m3::ASSET_BUDGET_BUDGET_RENDER_UI_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_ASSET_BUDGET_NAMING_CORE_ASSET_DYNAMIC_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::ASSET_BUDGET_NAMING_CORE_ASSET_DYNAMIC_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_ASSET_BUDGET_NAMING_GRAPHICS_MISC_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::ASSET_BUDGET_NAMING_GRAPHICS_MISC_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_ASSET_BUDGET_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = m3::ASSET_BUDGET_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_SCENE_SCRIPT_TESTS_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = m3::SCENE_SCRIPT_TESTS_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_SCENE_SCRIPT_TESTS_SCRIPT_VM_RUNTIME_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::SCENE_SCRIPT_TESTS_SCRIPT_VM_RUNTIME_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_SCENE_SCRIPT_TESTS_PLUGIN_EXTENSION_TESTS_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::SCENE_SCRIPT_TESTS_PLUGIN_EXTENSION_TESTS_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_SCENE_SCRIPT_TESTS_SCRIPT_VM_GAMEPLAY_SHADER_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::SCENE_SCRIPT_TESTS_SCRIPT_VM_GAMEPLAY_SHADER_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_SCENE_SCRIPT_TESTS_SCENE_ECS_TESTS_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::SCENE_SCRIPT_TESTS_SCENE_ECS_TESTS_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_SCENE_SCRIPT_TESTS_SCENE_ASSET_WORLD_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::SCENE_SCRIPT_TESTS_SCENE_ASSET_WORLD_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_SCENE_SCRIPT_TESTS_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::SCENE_SCRIPT_TESTS_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_STATUS_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = m3::STATUS_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_STATUS_SUPPORT_EXPECTED_SLICE_MAPS_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::STATUS_SUPPORT_EXPECTED_SLICE_MAPS_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_STATUS_SUPPORT_RUNTIME_INDEX_ANCHORS_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::STATUS_SUPPORT_RUNTIME_INDEX_ANCHORS_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_STATUS_SUPPORT_PRIORITY_PLAN_DOCS_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::STATUS_SUPPORT_PRIORITY_PLAN_DOCS_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_STATUS_SUPPORT_PRIORITY_PLAN_DOCS_OWNER_GUARDS_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::STATUS_SUPPORT_PRIORITY_PLAN_DOCS_OWNER_GUARDS_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_STATUS_SUPPORT_PRIORITY_PLAN_DOCS_OWNER_GUARDS_INVENTORY_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::STATUS_SUPPORT_PRIORITY_PLAN_DOCS_OWNER_GUARDS_INVENTORY_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_STATUS_SUPPORT_PRIORITY_PLAN_DOCS_STATUS_FOLLOWUPS_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::STATUS_SUPPORT_PRIORITY_PLAN_DOCS_STATUS_FOLLOWUPS_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_STATUS_SUPPORT_PRIORITY_PLAN_DOCS_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] =
    m3::STATUS_SUPPORT_PRIORITY_PLAN_DOCS_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_UI_TESTS_SECOND_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = m3::UI_TESTS_SECOND_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_PRODUCTION_GUARD_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = m3::PRODUCTION_GUARD_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES;
