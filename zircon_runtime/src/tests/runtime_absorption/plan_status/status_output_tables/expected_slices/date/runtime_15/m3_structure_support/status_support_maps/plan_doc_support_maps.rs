pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 status-support expected-slice map child split" => Some("2026-07-05"),
        "Runtime 15 M3 status output expected-slice legacy child-owner split" => Some("2026-06-24"),
        "Runtime 15 M3 status output expected-slice legacy group child-owner split" => {
            Some("2026-06-24")
        }
        "Runtime 15 M3 status output expected-slice guard child-owner split" => Some("2026-06-24"),
        "Runtime 15 M3 production guard support row-data child split" => Some("2026-07-04"),
        "Runtime 15 M3 production guard runtime row-data child split" => Some("2026-07-04"),
        "Runtime 15 M3 structure-support expected-slice map child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 structure-convention warning cleanup" => Some("2026-07-01"),
        "Runtime 15 M3 runtime index subplan map 01-15 sync" => Some("2026-07-01"),
        "Runtime 15 M3 runtime index problem-row parser P01-P17 sync" => Some("2026-07-01"),
        "Runtime 15 M3 Runtime 03 module-doc status index anchor sync" => Some("2026-07-01"),
        "Runtime 15 M3 Runtime 07 scene asset status anchor sync" => Some("2026-07-01"),
        "Runtime 15 M3 Runtime 07 owner-budget status anchor sync" => Some("2026-07-01"),
        "Runtime 15 M3 Runtime 02 generated status anchor sync" => Some("2026-07-01"),
        "Runtime 15 M3 Runtime 10 behavior status anchor sync" => Some("2026-07-01"),
        "Runtime 15 M3 Runtime Cargo attempt status anchor sync" => Some("2026-07-01"),
        "Runtime 15 M3 plan-status index-tables child-owner split" => Some("2026-07-01"),
        "Runtime 15 M3 plan-status support inventory review sync" => Some("2026-07-01"),
        "Runtime 15 M3 priority plan docs code-path integrity guard" => Some("2026-07-01"),
        "Runtime 15 M3 priority plan docs test-path integrity guard" => Some("2026-07-01"),
        "Runtime 15 M3 priority plan docs frontmatter status guard" => Some("2026-07-01"),
        "Runtime 15 M3 priority plan docs frontmatter uniqueness guard" => Some("2026-07-03"),
        "Runtime 15 M3 priority plan docs required header sections guard" => Some("2026-07-01"),
        "Runtime 15 M3 priority plan docs plan-source cross-link guard" => Some("2026-07-01"),
        "Runtime 15 M3 priority plan docs guard-test listing guard" => Some("2026-07-01"),
        "Runtime 15 M3 priority plan docs guard child-owner split" => Some("2026-07-01"),
        "Runtime 15 M3 priority plan docs child prose full inventory sync" => Some("2026-07-04"),
        "Runtime 15 M3 priority plan docs guard-test child-owner split" => Some("2026-07-01"),
        "Runtime 15 M3 priority plan docs guard-test child prose full inventory sync" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 priority plan docs moved guard path mirror" => Some("2026-07-01"),
        "Runtime 15 M3 priority plan docs guard inventory row-data source sync" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 priority plan docs listing prose full inventory sync" => Some("2026-07-04"),
        "Runtime 15 M3 priority plan docs moved mirror full inventory sync" => Some("2026-07-04"),
        "Runtime 15 M3 priority plan docs row-data owner child split" => Some("2026-07-02"),
        "Runtime 15 M3 priority plan docs owner-guard row-data child split" => Some("2026-07-04"),
        // Status: runtime_15_priority_plan_docs_row_data_owner_child_split_static_passed_cargo_deferred.
        // Files: plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/integrity_guards.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/owner_guards.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/status_followups.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/row_data_owner.rs.
        // Guard: runtime_15_priority_plan_docs_row_data_owner_is_child_backed.
        "Runtime 15 M3 priority plan docs row-data guard folder-backed split" => Some("2026-07-03"),
        "Runtime 15 M3 priority plan docs status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 priority plan docs root inventory child split" => Some("2026-07-04"),
        "Runtime 15 M3 status output owner stale-path follow-up" => Some("2026-07-01"),
        "Runtime 15 M3 status-support row-data owner child split" => Some("2026-07-02"),
        "Runtime 15 M3 status-support row-data guard folder-backed split" => Some("2026-07-03"),
        "Runtime 15 M3 status-support row-data status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 status-support row-data-and-budget child split" => Some("2026-07-04"),
        "Runtime 15 M3 status-support row-data root inventory child split" => Some("2026-07-04"),
        // Status: runtime_15_scene_script_row_data_owner_child_split_static_passed_cargo_deferred.
        // Files: plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/runtime_07_performance.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/script_vm_runtime.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/plugin_extension_tests.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/script_vm_gameplay_shader.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/scene_ecs_tests.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/scene_asset_world.rs.
        // Guard: runtime_15_scene_script_row_data_owner_is_child_backed.
        // Status: runtime_15_scene_script_row_data_guard_folder_backed_static_passed_cargo_deferred.
        // Guard: runtime_15_scene_script_row_data_guard_is_folder_backed.
        // Status: runtime_15_scene_script_row_data_status_mirror_child_split_static_passed_cargo_deferred.
        // Guard: runtime_15_scene_script_row_data_status_mirror_children_are_child_owned.
        // Status: runtime_15_scene_script_row_data_root_inventory_child_split_static_passed_cargo_deferred.
        // Guard: runtime_15_scene_script_row_data_root_inventory_is_child_owned.
        // Status: runtime_15_status_support_row_data_owner_child_split_static_passed_cargo_deferred.
        // Files: plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/runtime_index_anchors.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs.rs.
        // Guard: runtime_15_status_support_row_data_owner_is_child_backed.
        // Status: runtime_15_status_support_row_data_and_budget_child_split_static_passed_cargo_deferred.
        // Files: plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget/test_file_budget.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget/runtime_row_data.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget/hub_editor_support.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget/render_shader_support.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget/m3_m4_row_data.rs.
        // Guard: runtime_15_status_support_row_data_and_budget_children_are_child_owned.
        "Runtime 15 M3 asset-budget row-data owner child split" => Some("2026-07-02"),
        "Runtime 15 M3 asset-budget row-data guard folder-backed split" => Some("2026-07-03"),
        "Runtime 15 M3 asset-budget row-data status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 asset-budget row-data root inventory child split" => Some("2026-07-04"),
        // Status: runtime_15_asset_budget_row_data_owner_child_split_static_passed_cargo_deferred.
        // Files: plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/asset_budget_tests.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/asset_budget_tests/runtime_rhi.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/asset_budget_tests/asset_tests.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/asset_budget_tests/budget_render_ui.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/asset_budget_tests/naming_core_asset_dynamic.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/asset_budget_tests/naming_graphics_misc.rs.
        // Guard: runtime_15_asset_budget_row_data_owner_is_child_backed.
        "Runtime 15 M3 mesh pipeline shader source tests child-owner split" => Some("2026-07-01"),
        // Status: runtime_15_render_shader_template_assembly_guard_support_child_owner_split_static_passed_cargo_deferred.
        // Files: structure_convention/production_file_budget/render_shader_template_assembly.rs; structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions.rs; structure_convention/production_file_budget/render_shader_template_assembly/docs_anchors.rs; structure_convention/production_file_budget/render_shader_template_assembly/sources.rs.
        // Guard: runtime_15_render_shader_template_assembly_support_children_are_folder_backed.
        // Status: runtime_15_render_shader_template_assembly_assertion_contract_child_owner_split_static_passed_cargo_deferred.
        // Files: structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions.rs; structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions/template_contracts.rs; structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions/mesh_cache_contracts.rs; structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions/mesh_pipeline_shadow_graph_contracts.rs; structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions/owner_budget.rs.
        // Guard: runtime_15_render_shader_template_assembly_support_children_are_folder_backed.
        // Status: runtime_15_mesh_pipeline_shader_source_tests_child_owner_split_static_passed_cargo_deferred.
        // Files: graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs; graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests.rs; graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests/runtime_shading_model_sources.rs.
        // Guard: runtime_15_render_shader_template_assembly_support_children_are_folder_backed.
        // Status: runtime_15_plan_status_index_tables_child_owner_split_static_passed_cargo_deferred.
        // Files: plan_status/index_tables.rs; plan_status/index_tables/subplan_map.rs; plan_status/index_tables/status_anchors.rs; plan_status/index_tables/index_consistency.rs.
        // Guard: runtime_15_plan_status_index_tables_guard_child_owner_split.
        // Status: runtime_15_plan_status_support_inventory_review_sync_static_passed_cargo_deferred.
        // Status: runtime_15_runtime_cargo_attempt_status_anchor_sync_static_passed_cargo_deferred.
        // Guard: runtime_15_runtime_cargo_attempt_status_index_anchors_are_locked.
        // Status: runtime_15_runtime_10_behavior_status_anchor_sync_static_passed_cargo_deferred.
        // Guard: runtime_15_runtime_10_behavior_status_index_anchors_are_locked.
        // Status: runtime_15_runtime_02_generated_status_anchor_sync_static_passed_cargo_deferred.
        // Guard: runtime_15_runtime_02_generated_status_index_anchors_are_locked.
        // Status: runtime_15_runtime_07_owner_budget_status_anchor_sync_static_passed_cargo_deferred.
        // Guard: runtime_15_runtime_07_owner_budget_status_index_anchors_are_locked.
        // Status: runtime_15_runtime_07_scene_asset_status_anchor_sync_static_passed_cargo_deferred.
        // Guard: runtime_15_runtime_07_scene_asset_status_index_anchors_are_locked.
        // Status: runtime_15_runtime_03_module_doc_status_index_anchor_sync_static_passed_cargo_deferred.
        // Guard: runtime_15_runtime_03_module_doc_status_index_anchors_are_locked.
        // Status: runtime_15_runtime_index_problem_row_parser_p01_p17_sync_static_passed_cargo_deferred.
        // Guard: runtime_15_runtime_index_problem_row_parser_covers_p01_p17_status_locked.
        // Status: runtime_15_runtime_index_subplan_map_01_15_sync_static_passed_cargo_deferred.
        // Guard: runtime_15_runtime_index_subplan_map_covers_01_15_status_locked.
        _ => None,
    }
}
