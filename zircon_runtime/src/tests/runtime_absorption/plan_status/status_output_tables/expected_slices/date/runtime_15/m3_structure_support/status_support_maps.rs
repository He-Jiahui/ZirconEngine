pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 test file budget root-layout child split" => Some("2026-06-23"),
        "Runtime 15 M3 status output Runtime 15 row data split" => Some("2026-06-23"),
        "Runtime 15 M3 Runtime 15 row-data guard folder-backed split" => Some("2026-07-03"),
        "Runtime 15 M3 status output Runtime 15 foundation row data split" => Some("2026-06-27"),
        "Runtime 15 M3 foundation row-data topic child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 foundation row-data 73-row docs sync" => Some("2026-07-01"),
        "Runtime 15 M3 foundation row-data stale-count prose guard" => Some("2026-07-03"),
        "Runtime 15 M3 foundation row-data priority-doc frontmatter sync" => Some("2026-07-03"),
        "Runtime 15 M3 status output Runtime 15 M2 row data split" => Some("2026-06-29"),
        "Runtime 15 M3 M2 row-data guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 M2 row-data guard folder-backed split" => Some("2026-07-03"),
        "Runtime 15 M3 M2 row-data status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 support Hub project-actions tests child-owner split" => Some("2026-06-27"),
        "Runtime 15 M3 support Hub runtime-state tests child-owner split" => Some("2026-06-27"),
        "Runtime 15 M3 support Hub view-model quick-actions/tests child-owner split" => {
            Some("2026-06-27")
        }
        "Runtime 15 M3 editor retained-host workbench window projection tests child-owner split" => {
            Some("2026-06-27")
        }
        "Runtime 15 M3 editor retained-host pane data conversion projection owner guard" => {
            Some("2026-06-27")
        }
        "Runtime 15 M3 production file budget core runtime guard split" => Some("2026-06-23"),
        "Runtime 15 M3 render shader template assembly guard support child-owner split" => {
            Some("2026-06-27")
        }
        "Runtime 15 M3 render shader template assembly assertion contract child-owner split" => {
            Some("2026-07-01")
        }
        "Runtime 15 M3 shader prewarm manifest guard child-owner split" => Some("2026-06-27"),
        "Runtime 15 M3 status output Runtime 15 M4 row data split" => Some("2026-06-23"),
        "Runtime 15 M3 Runtime 15 M4 row-data guard folder-backed split" => Some("2026-07-03"),
        "Runtime 15 M3 status output expected-slice maps split" => Some("2026-06-23"),
        "Runtime 15 M3 status output Runtime 15 expected-slice child-owner split" => {
            Some("2026-06-25")
        }
        "Runtime 15 M3 status output expected-slice guard maps child-owner split" => Some("2026-06-25"),
        "Runtime 15 M3 status output expected-slice top-level map support child-owner split" => {
            Some("2026-06-25")
        }
        "Runtime 15 M3 status output Runtime 15 M3 row data split" => Some("2026-06-23"),
        "Runtime 15 M3 Runtime 15 M3 row-data guard folder-backed split" => Some("2026-07-03"),
        "Runtime 15 M3 test file budget root-layout status scan child split" => Some("2026-06-23"),
        "Runtime 15 M3 test file budget root-layout folder-backed guard child split" => {
            Some("2026-06-24")
        }
        "Runtime 15 M3 test file budget root-layout folder-backed support child-owner split" => {
            Some("2026-06-25")
        }
        "Runtime 15 M3 root-layout status-output Runtime 15 row-data child-source sync" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 test file budget parent guard child-owner split" => Some("2026-06-24"),
        "Runtime 15 M3 historical oversized test roots closeout" => Some("2026-06-23"),
        "Runtime 15 M3 asset test-budget guard child-owner split" => Some("2026-06-23"),
        "Runtime 15 M3 UI asset test folder split" => Some("2026-06-23"),
        "Runtime 15 M3 UI asset surface index test folder split" => Some("2026-06-24"),
        "Runtime 15 M3 UI asset MUI web form style test folder split" => Some("2026-06-24"),
        "Runtime 15 M3 UI asset MUI X web style test folder split" => Some("2026-06-23"),
        "Runtime 15 M3 UI asset MUI web style test folder split" => Some("2026-06-23"),
        "Runtime 15 M3 UI taffy layout pass test folder split" => Some("2026-06-23"),
        "Runtime 15 M3 UI runtime window input pump test folder split" => Some("2026-06-23"),
        "Runtime 15 M3 UI runtime window event ABI child folder split" => Some("2026-06-23"),
        "Runtime 15 M3 test file budget root-layout UI child split" => Some("2026-06-23"),
        "Runtime 15 M3 UI widget text input keyboard test folder split" => Some("2026-06-23"),
        "Runtime 15 M3 UI focus navigation test folder split" => Some("2026-06-23"),
        "Runtime 15 M3 UI runtime input manager test folder split" => Some("2026-06-23"),
        "Runtime 15 M3 UI runtime input ownership test folder split" => Some("2026-06-23"),
        "Runtime 15 M3 production file budget guard child-owner split" => Some("2026-06-24"),
        "Runtime 15 M3 status output variable evidence anchors" => Some("2026-06-24"),
        "Runtime 15 M3 status output evidence anchors guard folder-backed split" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 status output M3 row data child-owner split" => Some("2026-06-24"),
        "Runtime 15 M3 status output row-data guard child-owner split" => Some("2026-06-24"),
        "Runtime 15 M3 status-output row-data module-layout guard folder-backed split" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 module-layout status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 status output row-data module-layout status-doc guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 module-layout status-doc guard folder-backed split" => Some("2026-07-03"),
        "Runtime 15 M3 module-layout status-doc status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 status output row-data module-layout child-summary guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 module-layout child-summary guard folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 module-layout child-summary milestone-groups child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 module-layout child-summary foundation-review child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 module-layout child-summary status-doc guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 module-layout child-summary status-doc guard folder-backed split" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 module-layout child-summary status-doc status-mirror child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 status output review-guard row-data guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 review-guard row-data status-doc guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 review-guard row-data moved-row guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 review-guard moved-row guard folder-backed split" => Some("2026-07-02"),
        "Runtime 15 M3 review-guard moved-row status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 review-guard code-review row-data guard folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 review-guard code-review status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 review-guard row-data guard folder-backed split" => Some("2026-07-02"),
        "Runtime 15 M3 review-guard row-data status-doc guard folder-backed split" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 review-guard row-data status-doc status-mirror child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 review-guard direct-assertion row-data guard folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 review-guard direct-assertion status-mirror child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 review-guard direct-assertion row-data child-owner split" => {
            Some("2026-07-01")
        }
        "Runtime 15 M3 foundation row-data guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 foundation row-data guard folder-backed split" => Some("2026-07-02"),
        "Runtime 15 M3 foundation-guards row-data guard folder-backed split" => {
            Some("2026-07-03")
        }
        "Runtime 15 M3 foundation-guards row-data status-mirror child split" => {
            Some("2026-07-04")
        }
        "Runtime 15 M3 foundation row-data status-doc guard child-owner split" => {
            Some("2026-06-30")
        }
        "Runtime 15 M3 foundation row-data status-doc guard folder-backed split" => {
            Some("2026-07-02")
        }
        "Runtime 15 M3 foundation row-data row-count child split" => Some("2026-07-04"),
        "Runtime 15 M3 child-groups status-doc guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 child-groups status-doc guard folder-backed split" => Some("2026-07-03"),
        "Runtime 15 M3 child-groups status-doc status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 child-groups row-data guard folder-backed split" => Some("2026-07-03"),
        "Runtime 15 M3 child-groups row-data status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 child-group status-row-doc guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 child-group status-row-doc guard folder-backed split" => Some("2026-07-02"),
        "Runtime 15 M3 child-group status-row-doc status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 lock-poison status row-data guard folder-backed split" => Some("2026-07-02"),
        "Runtime 15 M3 lock-poison status row-data status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 scene-script row-data guard folder-backed split" => Some("2026-07-02"),
        "Runtime 15 M3 scene-script row-data status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 child-group moved-row guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 child-group moved-row guard folder-backed split" => Some("2026-07-03"),
        "Runtime 15 M3 status output expected-slice legacy child-owner split" => Some("2026-06-24"),
        "Runtime 15 M3 status output expected-slice legacy group child-owner split" => {
            Some("2026-06-24")
        }
        "Runtime 15 M3 status output expected-slice guard child-owner split" => Some("2026-06-24"),
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
        "Runtime 15 M3 priority plan docs guard inventory row-data source sync" => Some("2026-07-04"),
        "Runtime 15 M3 priority plan docs listing prose full inventory sync" => Some("2026-07-04"),
        "Runtime 15 M3 priority plan docs moved mirror full inventory sync" => Some("2026-07-04"),
        "Runtime 15 M3 priority plan docs row-data owner child split" => Some("2026-07-02"),
        "Runtime 15 M3 priority plan docs owner-guard row-data child split" => Some("2026-07-04"),
        // Status: runtime_15_priority_plan_docs_row_data_owner_child_split_static_passed_cargo_deferred.
        // Files: plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/integrity_guards.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/owner_guards.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/status_followups.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/row_data_owner.rs.
        // Guard: runtime_15_priority_plan_docs_row_data_owner_is_child_backed.
        "Runtime 15 M3 priority plan docs row-data guard folder-backed split" => Some("2026-07-03"),
        "Runtime 15 M3 priority plan docs status-mirror child split" => Some("2026-07-04"),
        "Runtime 15 M3 status output owner stale-path follow-up" => Some("2026-07-01"),
        "Runtime 15 M3 status-support row-data owner child split" => Some("2026-07-02"),
        "Runtime 15 M3 status-support row-data guard folder-backed split" => Some("2026-07-03"),
        // Status: runtime_15_status_support_row_data_owner_child_split_static_passed_cargo_deferred.
        // Files: plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/runtime_index_anchors.rs; plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs.rs.
        // Guard: runtime_15_status_support_row_data_owner_is_child_backed.
        "Runtime 15 M3 asset-budget row-data owner child split" => Some("2026-07-02"),
        "Runtime 15 M3 asset-budget row-data guard folder-backed split" => Some("2026-07-03"),
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
