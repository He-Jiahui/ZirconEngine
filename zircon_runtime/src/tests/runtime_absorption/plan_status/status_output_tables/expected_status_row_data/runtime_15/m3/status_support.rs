use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
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
        "Runtime 15 M3 status output Runtime 15 row data split",
        &[
            "runtime_15_status_output_runtime_15_row_data_split_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_status_row_data.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
            "runtime_15_status_output_runtime_15_row_data_is_child_owner",
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
        "Runtime 15 M3 status output expected-slice maps split",
        &[
            "runtime_15_status_output_expected_slice_maps_split_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_slices/status.rs",
            "plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
            "runtime_15_status_output_expected_slice_maps_are_child_owners",
        ],
    ),
    (
        "Runtime 15 M3 status output Runtime 15 expected-slice child-owner split",
        &[
            "runtime_15_status_output_runtime_15_expected_slice_child_owner_split_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs",
            "runtime_15_status_output_runtime_15_expected_slice_maps_are_child_owners",
        ],
    ),
    (
        "Runtime 15 M3 status output expected-slice guard maps child-owner split",
        &[
            "runtime_15_status_output_expected_slice_guard_maps_child_owner_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/status_output_expected_slices/maps.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics.rs",
            "runtime_15_status_output_expected_slice_guard_maps_are_child_owners",
        ],
    ),
    (
        "Runtime 15 M3 status output expected-slice top-level map support child-owner split",
        &[
            "runtime_15_status_output_expected_slice_top_level_map_support_child_owner_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/assertions.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/sources.rs",
            "runtime_15_status_output_expected_slice_top_level_map_support_child_owners_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 status output Runtime 15 M3 row data split",
        &[
            "runtime_15_status_output_runtime_15_m3_row_data_split_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs",
            "runtime_15_status_output_runtime_15_m3_row_data_is_child_owner",
        ],
    ),
];
