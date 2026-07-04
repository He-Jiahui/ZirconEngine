#[path = "review_guard_maps/typed_error_maps.rs"]
mod typed_error_maps;

pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if let Some(status) = typed_error_maps::expected_status_for_slice(slice) {
        return Some(status);
    }

    match slice {
        "Runtime 15 M3 code review findings test folder split" => {
            Some("runtime_15_code_review_findings_tests_folder_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code-review standalone harness current-path sync" => {
            Some("runtime_15_code_review_standalone_harness_current_path_sync_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 P0 robustness review guard child-owner split" => Some(
            "runtime_15_p0_robustness_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 P0 robustness structure guard folder-backed split" => Some(
            "runtime_15_p0_robustness_structure_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 P0 robustness root inventory child split" => Some(
            "runtime_15_p0_robustness_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 P0 route ownership guard child split" => Some(
            "runtime_15_p0_route_ownership_guard_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 P0 native fixture review guard leaf-owner split" => Some(
            "runtime_15_p0_native_fixture_review_guard_leaf_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 F8 API convergence review guard child-owner split" => Some(
            "runtime_15_f8_api_convergence_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 F8 child-owner structure guard folder-backed split" => Some(
            "runtime_15_f8_child_owner_structure_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 F8 child-owner root inventory child split" => Some(
            "runtime_15_f8_child_owner_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 F8 route ownership guard child split" => Some(
            "runtime_15_f8_route_ownership_guard_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 F8 descriptor review guard child-owner split" => Some(
            "runtime_15_f8_descriptor_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 late API cleanup review guard child-owner split" => Some(
            "runtime_15_late_api_cleanup_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 late API cleanup structure guard folder-backed split" => Some(
            "runtime_15_late_api_cleanup_structure_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 late API cleanup root inventory child split" => Some(
            "runtime_15_late_api_cleanup_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings structure guard child-owner split" => Some(
            "runtime_15_code_review_findings_structure_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings structure guard children folder-backed split" => Some(
            "runtime_15_code_review_findings_structure_guard_children_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings structure guard children budget-status child split" => {
            Some("runtime_15_code_review_findings_structure_guard_children_budget_status_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings structure guard children root inventory child split" => {
            Some("runtime_15_code_review_findings_structure_guard_children_root_inventory_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 structure guard plugin-importer child split" => Some(
            "runtime_15_structure_guard_plugin_importer_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 P0 native fixture leaf-owner guard folder-backed split" => Some(
            "runtime_15_p0_native_fixture_leaf_owner_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 P0 native fixture leaf-owner root inventory child split" => Some(
            "runtime_15_p0_native_fixture_leaf_owner_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings status-doc guard child-owner split" => Some(
            "runtime_15_code_review_findings_status_docs_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings status-doc guard folder-backed split" => Some(
            "runtime_15_code_review_findings_status_docs_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings status-doc status-mirror child-owner split" => {
            Some("runtime_15_code_review_findings_status_docs_status_mirror_child_owner_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings status-doc source anchors child-owner split" => Some(
            "runtime_15_code_review_findings_status_docs_source_anchors_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings status-doc source anchors folder-backed split" => Some(
            "runtime_15_code_review_findings_status_docs_source_anchors_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings status-doc status anchors child-owner split" => Some(
            "runtime_15_code_review_findings_status_docs_status_anchors_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings status-doc status anchors folder-backed split" => Some(
            "runtime_15_code_review_findings_status_docs_status_anchors_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings status-doc child-anchor list child split" => Some(
            "runtime_15_code_review_findings_status_docs_child_anchor_list_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings status-doc root inventory child split" => Some(
            "runtime_15_code_review_findings_status_docs_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings status-doc status anchor guard folder-backed split" => {
            Some("runtime_15_code_review_findings_status_docs_status_anchor_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings status-doc status-anchor child-ownership child split" => {
            Some("runtime_15_code_review_findings_status_docs_status_anchor_child_ownership_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings folder-backed summary child-owner split" => Some(
            "runtime_15_code_review_findings_folder_backed_summary_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings folder-backed summary guard folder-backed split" => Some(
            "runtime_15_code_review_findings_folder_backed_summary_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings folder-backed summary child-ownership guard folder-backed split" => {
            Some("runtime_15_code_review_findings_folder_backed_summary_child_ownership_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings source inventory child-owner split" => Some(
            "runtime_15_code_review_findings_source_inventory_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings source inventory folder-backed split" => Some(
            "runtime_15_code_review_findings_source_inventory_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings source inventory status-mirror child-owner split" => {
            Some("runtime_15_code_review_findings_source_inventory_status_mirror_child_owner_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings direct assertions child-owner split" => Some(
            "runtime_15_code_review_findings_direct_assertions_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings direct assertions guard folder-backed split" => {
            Some("runtime_15_code_review_findings_direct_assertions_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings direct assertions child-ownership guard folder-backed split" => {
            Some("runtime_15_code_review_findings_direct_assertions_child_ownership_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings F12 direct assertions child-owner split" => Some(
            "runtime_15_code_review_findings_f12_direct_assertions_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings F12 direct assertions guard folder-backed split" => {
            Some("runtime_15_code_review_findings_f12_direct_assertions_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings root-parent direct assertions child-owner split" => {
            Some("runtime_15_code_review_findings_root_parent_direct_assertions_child_owner_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings root-parent direct assertions guard folder-backed split" => {
            Some("runtime_15_code_review_findings_root_parent_direct_assertions_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings render direct assertions child-owner split" => {
            Some("runtime_15_code_review_findings_render_direct_assertions_child_owner_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings render direct assertions guard folder-backed split" => {
            Some("runtime_15_code_review_findings_render_direct_assertions_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings F8 direct assertions child-owner split" => Some(
            "runtime_15_code_review_findings_f8_direct_assertions_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings F8 direct assertions guard folder-backed split" => {
            Some("runtime_15_code_review_findings_f8_direct_assertions_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings P0 direct assertions child-owner split" => Some(
            "runtime_15_code_review_findings_p0_direct_assertions_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings P0 direct assertions guard folder-backed split" => {
            Some("runtime_15_code_review_findings_p0_direct_assertions_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings structure guard folder-backed summary child-owner split" => {
            Some("runtime_15_code_review_findings_structure_guard_folder_backed_summary_child_owner_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings structure guard folder-backed summary guard folder-backed split" => {
            Some("runtime_15_code_review_findings_structure_guard_folder_backed_summary_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings structure guard typed-error child-owner split" => {
            Some("runtime_15_code_review_findings_structure_guard_typed_error_child_owner_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings structure guard typed-error folder-backed split" => {
            Some("runtime_15_code_review_findings_structure_guard_typed_error_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings typed-error structure guard child-owner split" => Some(
            "runtime_15_code_review_findings_typed_error_structure_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error structure guard folder-backed split" => Some(
            "runtime_15_typed_error_structure_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error structure guard root inventory child split" => Some(
            "runtime_15_typed_error_structure_guard_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error child-ownership guard folder-backed split" => Some(
            "runtime_15_typed_error_child_ownership_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error child-ownership root inventory child split" => Some(
            "runtime_15_typed_error_child_ownership_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error structure assertions guard child-owner split" => Some(
            "runtime_15_typed_error_structure_assertions_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error structure assertions guard folder-backed split" => Some(
            "runtime_15_typed_error_structure_assertions_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error convergence mounts guard folder-backed split" => Some(
            "runtime_15_typed_error_convergence_mounts_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error convergence mounts root inventory child split" => Some(
            "runtime_15_typed_error_convergence_mounts_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error native plugin loader structure guard child-owner split" => {
            Some("runtime_15_typed_error_native_plugin_loader_structure_guard_child_owner_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error native plugin loader structure guard folder-backed split" => {
            Some("runtime_15_typed_error_native_plugin_loader_structure_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error native plugin loader routes child split" => {
            Some("runtime_15_typed_error_native_plugin_loader_routes_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error native plugin loader routes source helper child split" => {
            Some("runtime_15_typed_error_native_plugin_loader_routes_source_helper_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error native plugin loader source helper child split" => {
            Some("runtime_15_typed_error_native_plugin_loader_source_helper_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error structure moved-guard absence child-owner split" => Some(
            "runtime_15_typed_error_structure_moved_guard_absence_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error moved-guard absence guard folder-backed split" => Some(
            "runtime_15_typed_error_moved_guard_absence_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error moved-guard absence root inventory child split" => Some(
            "runtime_15_typed_error_moved_guard_absence_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings plugin-importer DX structure guard child-owner split" => Some(
            "runtime_15_code_review_findings_plugin_importer_dx_structure_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 plugin-importer DX structure guard folder-backed split" => Some(
            "runtime_15_plugin_importer_dx_structure_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 plugin-importer DX structure guard root inventory child split" => Some(
            "runtime_15_plugin_importer_dx_structure_guard_root_inventory_child_split_target_server_direct_binary_passed",
        ),
        "Runtime 15 M3 plugin-importer DX status-doc guard child-owner split" => Some(
            "runtime_15_plugin_importer_dx_status_docs_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 plugin-importer DX status-doc guard folder-backed split" => Some(
            "runtime_15_plugin_importer_dx_status_docs_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 plugin-importer DX status-doc root inventory child split" => Some(
            "runtime_15_plugin_importer_dx_status_docs_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 plugin-importer DX source inventory guard child-owner split" => Some(
            "runtime_15_plugin_importer_dx_source_inventory_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 plugin-importer DX source inventory guard folder-backed split" => Some(
            "runtime_15_plugin_importer_dx_source_inventory_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 plugin-importer DX structure assertions guard child-owner split" => Some(
            "runtime_15_plugin_importer_dx_structure_assertions_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 plugin-importer DX structure assertions guard folder-backed split" => {
            Some("runtime_15_plugin_importer_dx_structure_assertions_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 plugin-importer DX review mounts guard folder-backed split" => {
            Some("runtime_15_plugin_importer_dx_review_mounts_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 plugin-importer D13 SDK structure assertions guard folder-backed split" => {
            Some("runtime_15_plugin_importer_d13_sdk_structure_assertions_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 plugin-importer D13 SDK parent-mount guard child split" => {
            Some("runtime_15_plugin_importer_d13_sdk_parent_mounts_guard_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 plugin-importer D13 SDK structure assertions guard child-owner split" => {
            Some("runtime_15_plugin_importer_d13_sdk_structure_assertions_guard_child_owner_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 plugin-importer DX review guard child-owner split" => Some(
            "runtime_15_plugin_importer_dx_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 plugin-importer D13 SDK review guard child-owner split" => Some(
            "runtime_15_plugin_importer_d13_sdk_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review top-row status row-data child-owner split" => Some(
            "runtime_15_review_top_row_status_row_data_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 D-S7 static plugin manifest generation/parity review sync" => {
            Some("ds7_static_plugin_manifest_generation_parity_review_synced_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 D7 core workspace dependency top-row closed status sync" => {
            Some("d7_core_workspace_dependency_top_row_closed_status_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 D7 core workspace dependency inheritance guard" => {
            Some("d7_core_workspace_dependency_inheritance_guard_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 D8 runtime registration builder original evidence paths" => {
            Some("d8_runtime_registration_builder_original_paths_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 D6 RuntimePluginId open string-newtype review sync" => {
            Some("d6_runtime_plugin_id_open_string_newtype_review_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 F5/F6/F7 typed-error top-row closed status sync" => {
            Some("f5_f6_f7_typed_error_top_row_closed_status_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 F8/F9/F10 runtime surface top-row closed status sync" => {
            Some("f8_f9_f10_runtime_surface_top_row_closed_status_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 F13/F14 provider diagnostics top-row closed status sync" => {
            Some("f13_f14_provider_diagnostics_top_row_closed_status_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 F17/F18 lookup/manager top-row closed status sync" => {
            Some("f17_f18_lookup_manager_top_row_closed_status_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 F19 scene renderer construction top-row closed status sync" => {
            Some("f19_scene_renderer_construction_top_row_closed_status_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 D9 editor/runtime mirror consumer guard" => {
            Some("d9_editor_runtime_mirror_consumers_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 D5 editor authoring macro consumer guard" => {
            Some("d5_editor_authoring_macro_consumers_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 D12 runtime helper export macro review sync" => {
            Some("d12_runtime_export_macro_review_synced_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 D1 capability single-source review sync" => {
            Some("d1_capability_single_source_review_synced_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 D10 animation/physics bridge call migration" => {
            Some("d10_animation_physics_bridge_call_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 D11 animation/physics TestRuntime fixture migration" => {
            Some("d11_animation_physics_test_runtime_fixture_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 D13 importer manifest parity guard" => {
            Some("d13_importer_manifest_parity_guard_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 P0/DX priority D13 parity sync" => {
            Some("review_priority_recommendation_d13_parity_sync_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 D13 importer top-row closed status sync" => {
            Some("d13_importer_top_row_closed_status_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 D-S8/D3 native fixture top-row closed status sync" => {
            Some("ds8_d3_native_fixture_top_row_closed_status_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 P0 F1/F2/F4 top-row closed status sync" => {
            Some("p0_f1_f2_f4_top_row_closed_status_static_passed_cargo_deferred")
        }
        _ => None,
    }
}

// Runtime 15 M3 code-review row-data owner child split anchor mirror:
// runtime_15_code_review_rows_row_data_owner_child_split_static_passed_cargo_deferred
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows.rs
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/review_guard_rows.rs
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows.rs
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows.rs
// runtime_15_code_review_rows_row_data_owner_is_child_backed
// Runtime 15 M3 plugin-importer row-data owner child split anchor mirror:
// runtime_15_plugin_importer_row_data_owner_child_split_static_passed_cargo_deferred
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows.rs
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows/review_guards.rs
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows/status_docs.rs
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows/source_inventory.rs
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows/structure_assertions.rs
// plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows/row_data_owner.rs
// runtime_15_plugin_importer_rows_row_data_owner_is_child_backed
