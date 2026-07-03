pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 M4 row-data owner child split" {
        Some("2026-07-02")
        // Status: runtime_15_m4_row_data_owner_child_split_static_passed_cargo_deferred.
        // Files: plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m4/core_rhi_dynamic.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m4/asset_scene_render.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m4/ui_text_template.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m4/ui_surface_plugin.rs.
        // Guard: runtime_15_m4_row_data_owner_is_child_backed.
    } else if slice == "Runtime 15 M3 M4 row-data children guard folder-backed split" {
        Some("2026-07-03")
        // Status: runtime_15_m4_row_data_children_guard_folder_backed_static_passed_cargo_deferred.
        // Files: structure_convention/test_file_budget/status_output_row_data/runtime_15_m4_row_data_children.rs; structure_convention/test_file_budget/status_output_row_data/runtime_15_m4_row_data_children/delegation.rs; structure_convention/test_file_budget/status_output_row_data/runtime_15_m4_row_data_children/row_ownership.rs; structure_convention/test_file_budget/status_output_row_data/runtime_15_m4_row_data_children/status_mirrors.rs; structure_convention/test_file_budget/status_output_row_data/runtime_15_m4_row_data_children/budgets.rs.
        // Guard: runtime_15_m4_row_data_children_guard_is_folder_backed.
    } else if slice == "Runtime 15 M4 no oversized production files global gate" {
        Some("2026-06-30")
    } else if slice == "Runtime 15 M4 core runtime service-list owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 RHI WGPU command validation render-state owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 RHI WGPU device command-list owner split" {
        Some("2026-07-01")
    } else if slice == "Runtime 15 M4 RHI WGPU UI surface render/setup owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 RHI WGPU UI surface geometry test owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 RHI device handle owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 dynamic API session profile owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 dynamic API session registry owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 dynamic API shader prewarm tests owner split" {
        Some("2026-07-01")
    } else if slice == "Runtime 15 M4 native host API adapter tests owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 plugin bridge table diagnostics owner split" {
        Some("2026-07-01")
    } else if slice == "Runtime 15 M4 material asset value/readiness helper owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 material asset management record owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 asset artifact cache UI document owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 asset artifact cache material/shader owner split" {
        Some("2026-07-03")
    } else if slice == "Runtime 15 M4 mesh asset management record owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 asset project scan/import source collection owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 glTF labeled material subasset owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 texture descriptor settings parser owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 scene world render light collection owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 scene component lighting/post-process owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M3 render shader template assembly guard WGSL contracts split" {
        Some("2026-06-24")
    } else if slice
        == "Runtime 15 M4 core runtime render-stats graph execution-resources owner split"
    {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 render-stats product diagnostics test owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 extend pending draws material-input owner split" {
        // Status: runtime_15_extend_pending_draws_material_inputs_owner_split_static_passed_cargo_deferred.
        // Files: graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs.
        // Files: graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance/material_inputs.rs.
        // Guard: runtime_15_extend_pending_draws_tests_are_child_owner.
        Some("2026-07-03")
    } else if slice == "Runtime 15 M4 non-Base mesh variant render-call guard sync" {
        // Status: runtime_15_non_base_mesh_variant_render_call_guard_sync_static_passed_cargo_deferred.
        // Files: graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs.
        // Files: graphics/scene/scene_renderer/temporal/velocity/execute_velocity_object.rs.
        // Guard: runtime_15_non_base_mesh_variant_cache_owner_is_wired.
        Some("2026-07-03")
    } else if slice == "Runtime 15 M4 shader prewarm manifest path helper owner split" {
        // Status: runtime_15_shader_prewarm_manifest_path_helpers_owner_split_static_passed_cargo_deferred.
        // Files: bin/zircon_shader_prewarm/manifest.rs.
        // Files: bin/zircon_shader_prewarm/manifest/paths.rs.
        // Guards: runtime_15_no_oversized_production_files; runtime_15_shader_prewarm_manifest_tests_are_folder_backed.
        Some("2026-07-03")
    } else if slice == "Runtime 15 M4 shader prewarm owner guard sync" {
        // Status: runtime_15_shader_prewarm_owner_guard_sync_static_passed_cargo_deferred.
        // Files: tools/zircon_build_plugin_assets.py; tools/zircon_build_plugin_shader_descriptors.py.
        // Files: graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline/tests.rs.
        // Guards: shader-prewarm plugin/registry/product staged-cache Runtime 15 structure guards.
        Some("2026-07-03")
    } else if slice == "Runtime 15 M4 deferred GBuffer template output guard sync" {
        // Status: runtime_15_deferred_gbuffer_template_output_guard_sync_static_passed_cargo_deferred.
        // Files: graphics/shader/wgsl/zr_template_deferred_gbuffer.wgsl.
        // Files: graphics/shader/wgsl/zr_surface_types.wgsl.
        // Guard: runtime_15_deferred_gbuffer_pipeline_template_cache_is_mesh_cache_owned.
        Some("2026-07-03")
    } else if slice == "Runtime 15 M4 scene fixed light reflection write-field owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 scene world property-access physics write owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 scene world property-access physics entry owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 scene world project I/O mesh owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 UI text layout engine visual-order owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 font database descriptor helper owner split" {
        Some("2026-06-30")
    } else if slice == "Runtime 15 M4 screen-space UI text font-id report owner split" {
        Some("2026-06-30")
    } else if slice == "Runtime 15 M4 screen-space UI text font-id report visibility sync" {
        // Status: runtime_15_screen_space_ui_text_font_id_report_visibility_sync_static_passed_cargo_deferred.
        // File: graphics/scene/scene_renderer/ui/text/font_id_report.rs.
        // Guard: runtime_15_screen_space_ui_text_font_id_report_is_child_owner.
        Some("2026-07-03")
    } else if slice == "Runtime 15 M4 screen-space UI text tests owner split" {
        // Status-map anchor inventory for the screen-space UI text tests owner split.
        // Status: runtime_15_screen_space_ui_text_tests_owner_split_static_passed_cargo_deferred.
        // Files: graphics/scene/scene_renderer/ui/text.rs; graphics/scene/scene_renderer/ui/text/tests.rs.
        // Guard: runtime_15_screen_space_ui_text_tests_are_child_owner_split.
        Some("2026-07-03")
    } else if slice == "Runtime 15 M4 screen-space UI text tests status-map anchor sync" {
        // Status: runtime_15_screen_space_ui_text_tests_status_map_anchor_sync_static_passed_cargo_deferred.
        // Sync slice: expected_slices/status/runtime_15/m4_surface_cleanup.rs.
        // Sync slice: expected_slices/date/runtime_15/m4_surface_cleanup.rs.
        // Files: graphics/scene/scene_renderer/ui/text.rs; graphics/scene/scene_renderer/ui/text/tests.rs.
        // Guard: runtime_15_screen_space_ui_text_tests_are_child_owner_split.
        Some("2026-07-03")
    } else if slice == "Runtime 15 M4 SDF atlas/render tests folder-backed guard sync" {
        // Status: runtime_15_sdf_atlas_render_tests_folder_backed_guard_sync_static_passed_cargo_deferred.
        // Files: graphics/scene/scene_renderer/ui/sdf_atlas/tests/{mod.rs,plan.rs,allocation.rs,cache_report.rs,owner.rs}.
        // Files: graphics/scene/scene_renderer/ui/sdf_render/vertices.rs; graphics/scene/scene_renderer/ui/sdf_render/tests/{mod.rs,draw_plan.rs,shader_contract.rs,layout_placement.rs,prepare_report.rs}.
        // Guards: runtime_15_screen_space_ui_sdf_atlas_tests_are_child_owner_split; runtime_15_screen_space_ui_sdf_render_tests_are_child_owner_split.
        Some("2026-07-03")
    } else if slice == "Runtime 15 M4 UI layout arrange grid/masonry owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 UI dispatch input manager test owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 UI template MUI X DataGrid class owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 UI template document validation owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 UI template style slot-contract owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 UI v2 style runtime-state owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 UI v2 style token-resolution owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 UI accessibility extract state owner split" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 M4 UI component catalog editor-showcase helper owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 UI component state-reducer keyboard menu submenu owner split"
    {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 UI component state-reducer tree view editing owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 UI surface event-routing owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 UI surface property mutation metadata dirty owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 UI surface render feedback command/color owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 UI surface default-interactions keyboard/timer owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 M4 UI surface table column helper owner split" {
        Some("2026-06-24")
    } else if slice == "Runtime 15 F12 offscreen target texture owner cleanup" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 F12 render backend state owner cleanup" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 F12 gpu texture resource owner cleanup" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 F12 gpu material uniform owner cleanup" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 F12 gpu mesh order signature cleanup" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 F12 gpu model identity cleanup" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 F12 post-process LUT texture owner cleanup" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 F12 output target texture owner cleanup" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 F12 material runtime capture seed cleanup" {
        Some("2026-06-22")
    } else if slice == "Runtime 15 F12 resource streamer diagnostics accessor cleanup" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 F12 ResourceStreamer material diagnostics child owner split" {
        Some("2026-07-03")
    } else if slice == "Runtime 15 F12 resource streamer resolve texture id cleanup" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 F12 particle GPU readback output accessor cleanup" {
        Some("2026-06-23")
    } else if slice == "Runtime 15 F12 advanced plugin output test accessor cleanup" {
        Some("2026-06-23")
    } else {
        None
    }
}
