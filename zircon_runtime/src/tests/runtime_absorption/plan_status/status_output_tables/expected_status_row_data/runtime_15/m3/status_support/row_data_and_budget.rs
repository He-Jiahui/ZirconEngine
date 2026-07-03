type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 test file budget root-layout child split",
        &[
            "runtime_15_test_file_budget_root_layout_child_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/mod.rs",
            "structure_convention/test_file_budget/root_layout.rs",
            "runtime_15_test_file_budget_guard_is_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 test file budget root-layout status scan child split",
        &[
            "runtime_15_test_file_budget_root_layout_status_scan_child_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/root_layout.rs",
            "structure_convention/test_file_budget/root_layout/status_scan.rs",
            "runtime_15_test_file_budget_root_layout_status_scan_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 test file budget root-layout folder-backed guard child split",
        &[
            "runtime_15_test_file_budget_root_layout_folder_backed_guard_child_split_static_passed_cargo_timeout_no_result",
            "structure_convention/test_file_budget/root_layout.rs",
            "structure_convention/test_file_budget/root_layout/folder_backed.rs",
            "structure_convention/test_file_budget/root_layout/module_layout.rs",
            "runtime_15_test_file_budget_root_layout_folder_backed_guard_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 test file budget root-layout folder-backed support child-owner split",
        &[
            "runtime_15_test_file_budget_root_layout_folder_backed_support_child_owner_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/root_layout/folder_backed.rs",
            "structure_convention/test_file_budget/root_layout/folder_backed/assertions.rs",
            "structure_convention/test_file_budget/root_layout/folder_backed/guard_names.rs",
            "runtime_15_test_file_budget_root_layout_folder_backed_support_child_owners_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 root-layout status-output Runtime 15 row-data child-source sync",
        &[
            "runtime_15_root_layout_status_output_runtime_15_row_data_child_source_sync_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/root_layout/folder_backed.rs",
            "structure_convention/test_file_budget/root_layout/folder_backed/sources.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/row_ownership.rs",
            "runtime_15_test_file_budget_guard_is_folder_backed",
            "runtime_15_status_output_runtime_15_row_data_is_child_owner",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 test file budget parent guard child-owner split",
        &[
            "runtime_15_test_file_budget_parent_guard_child_owner_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/mod.rs",
            "structure_convention/test_file_budget/core_framework.rs",
            "structure_convention/test_file_budget/ui_v2_asset.rs",
            "structure_convention/test_file_budget/ui_shared_core.rs",
            "structure_convention/test_file_budget/module_layout.rs",
            "runtime_15_test_file_budget_parent_guard_child_owner_split",
        ],
    ),
    (
        "Runtime 15 M3 historical oversized test roots closeout",
        &[
            "runtime_15_historical_oversized_test_roots_closeout_static_passed_cargo_deferred",
            "core/framework/tests.rs",
            "ui/tests/v2_asset.rs",
            "runtime_15_historical_oversized_test_roots_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 foundation row-data 73-row docs sync",
        &[
            "runtime_15_foundation_row_data_71_row_docs_sync_static_passed_cargo_deferred",
            "21/23/18/11",
            "73 Runtime 15 foundation status rows",
            "runtime_15_foundation_row_data_docs_record_current_row_count",
        ],
    ),
    (
        "Runtime 15 M3 foundation row-data stale-count prose guard",
        &[
            "runtime_15_foundation_row_data_stale_count_prose_guard_static_passed_cargo_deferred",
            "runtime_15_foundation_row_data_docs_record_current_row_count",
            "stale Runtime 15 foundation row-count prose",
            "current 73-row foundation row-data facts",
            "73 Runtime 15 foundation status rows",
        ],
    ),
    (
        "Runtime 15 M3 status output Runtime 15 row data split",
        &[
            "runtime_15_status_output_runtime_15_row_data_split_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_status_row_data.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
            "runtime_15_status_output_runtime_15_row_data_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 Runtime 15 row-data guard folder-backed split",
        &[
            "runtime_15_runtime_15_row_data_guard_folder_backed_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/delegation.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/row_ownership.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/status_mirrors.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/budgets.rs",
            "runtime_15_runtime_15_row_data_guard_is_folder_backed",
            "runtime_15_status_output_runtime_15_row_data_is_child_owner",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 status output Runtime 15 foundation row data split",
        &[
            "runtime_15_status_output_runtime_15_foundation_row_data_split_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
            "runtime_15_status_output_runtime_15_foundation_row_data_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 foundation row-data topic child-owner split",
        &[
            "runtime_15_foundation_row_data_topic_child_owner_split_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation/core_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation/typed_error_runtime_rows.rs",
            "runtime_15_status_output_runtime_15_foundation_row_data_is_child_owner",
            "Cargo gate deferred",
        ],
    ),
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
        "Runtime 15 M3 support Hub project-actions tests child-owner split",
        &[
            "runtime_15_support_hub_project_actions_tests_child_owner_split_static_passed_cargo_deferred",
            "zircon_hub/src/tauri_app/runtime_state/project_actions.rs",
            "zircon_hub/src/tauri_app/runtime_state/project_actions/tests.rs",
            "runtime_15_support_hub_project_actions_tests_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 support Hub runtime-state tests child-owner split",
        &[
            "runtime_15_support_hub_runtime_state_tests_child_owner_split_static_passed_cargo_deferred",
            "zircon_hub/src/tauri_app/runtime_state.rs",
            "zircon_hub/src/tauri_app/runtime_state/tests.rs",
            "runtime_15_support_hub_runtime_state_tests_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 support Hub view-model quick-actions/tests child-owner split",
        &[
            "runtime_15_support_hub_view_model_quick_actions_tests_child_owner_split_static_passed_cargo_deferred",
            "zircon_hub/src/tauri_app/view_model.rs",
            "zircon_hub/src/tauri_app/view_model/quick_actions.rs",
            "zircon_hub/src/tauri_app/view_model/tests.rs",
            "runtime_15_support_hub_view_model_quick_actions_tests_are_child_owners",
        ],
    ),
    (
        "Runtime 15 M3 editor retained-host workbench window projection tests child-owner split",
        &[
            "runtime_15_editor_retained_host_workbench_window_projection_tests_child_owner_split_static_passed_cargo_deferred",
            "zircon_editor/src/ui/retained_host/ui/workbench_window_projection.rs",
            "zircon_editor/src/ui/retained_host/ui/workbench_window_projection/tests.rs",
            "runtime_15_editor_retained_host_workbench_window_projection_tests_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 editor retained-host pane data conversion projection owner guard",
        &[
            "runtime_15_editor_retained_host_pane_data_conversion_owner_guard_static_passed_cargo_deferred",
            "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs",
            "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/template_node_projection.rs",
            "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/animation_projection.rs",
            "zircon_editor/src/ui/retained_host/ui/apply_presentation/pane_conversion.rs",
            "runtime_15_editor_retained_host_pane_data_conversion_uses_child_projection_owners",
        ],
    ),
    (
        "Runtime 15 M3 production file budget core runtime guard split",
        &[
            "runtime_15_production_file_budget_core_runtime_guard_split_static_passed_cargo_deferred",
            "structure_convention/production_file_budget.rs",
            "structure_convention/production_file_budget/core_runtime_service_lists.rs",
            "runtime_15_production_file_budget_core_runtime_guard_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 render shader template assembly guard support child-owner split",
        &[
            "runtime_15_render_shader_template_assembly_guard_support_child_owner_split_static_passed_cargo_deferred",
            "structure_convention/production_file_budget/render_shader_template_assembly.rs",
            "structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions.rs",
            "structure_convention/production_file_budget/render_shader_template_assembly/docs_anchors.rs",
            "structure_convention/production_file_budget/render_shader_template_assembly/sources.rs",
            "runtime_15_render_shader_template_assembly_support_children_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 render shader template assembly assertion contract child-owner split",
        &[
            "runtime_15_render_shader_template_assembly_assertion_contract_child_owner_split_static_passed_cargo_deferred",
            "structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions.rs",
            "structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions/template_contracts.rs",
            "structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions/mesh_cache_contracts.rs",
            "structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions/mesh_pipeline_shadow_graph_contracts.rs",
            "structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions/owner_budget.rs",
            "runtime_15_render_shader_template_assembly_support_children_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 mesh pipeline shader source tests child-owner split",
        &[
            "runtime_15_mesh_pipeline_shader_source_tests_child_owner_split_static_passed_cargo_deferred",
            "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs",
            "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests.rs",
            "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests/runtime_shading_model_sources.rs",
            "runtime_15_render_shader_template_assembly_support_children_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 shader prewarm manifest guard child-owner split",
        &[
            "runtime_15_shader_prewarm_manifest_guard_child_owner_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/shader_prewarm_manifest.rs",
            "structure_convention/test_file_budget/shader_prewarm_manifest/manifest_contract.rs",
            "structure_convention/test_file_budget/shader_prewarm_manifest/geometry_source.rs",
            "structure_convention/test_file_budget/shader_prewarm_manifest/builtin_template_source.rs",
            "structure_convention/test_file_budget/shader_prewarm_manifest/asset_revision.rs",
            "runtime_15_shader_prewarm_manifest_guard_children_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 status output Runtime 15 M4 row data split",
        &[
            "runtime_15_status_output_runtime_15_m4_row_data_split_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs",
            "runtime_15_status_output_runtime_15_m4_row_data_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 Runtime 15 M4 row-data guard folder-backed split",
        &[
            "runtime_15_m4_row_data_guard_folder_backed_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m4_row_data.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m4_row_data/delegation.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m4_row_data/row_ownership.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m4_row_data/status_mirrors.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m4_row_data/budgets.rs",
            "runtime_15_m4_row_data_guard_is_folder_backed",
            "runtime_15_status_output_runtime_15_m4_row_data_is_child_owner",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 Runtime 15 M3 row-data guard folder-backed split",
        &[
            "runtime_15_m3_row_data_guard_folder_backed_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_row_data.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_row_data/delegation.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_row_data/row_ownership.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_row_data/status_mirrors.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_row_data/budgets.rs",
            "runtime_15_m3_row_data_guard_is_folder_backed",
            "runtime_15_status_output_runtime_15_m3_row_data_is_child_owner",
            "Cargo gate deferred",
        ],
    ),
];
