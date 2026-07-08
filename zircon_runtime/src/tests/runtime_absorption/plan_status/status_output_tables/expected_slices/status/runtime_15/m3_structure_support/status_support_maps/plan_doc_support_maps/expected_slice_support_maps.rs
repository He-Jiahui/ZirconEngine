pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 status-support plan-doc expected-slice maps folder-backed split" => Some(
            "runtime_15_status_support_plan_doc_expected_slice_maps_folder_backed_static_passed_cargo_deferred",
        ),
        // Cargo gate blocked by render environment exports.
        "Runtime 15 M3 status-support expected-slice map child split" => Some(
            "runtime_15_status_support_expected_slice_map_child_split_static_passed_cargo_blocked_render_environment_exports",
        ),
        "Runtime 15 M3 status output expected-slice legacy child-owner split" => Some(
            "runtime_15_status_output_expected_slice_legacy_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status-output expected-slice legacy guard body folder-backed split" => Some(
            "runtime_15_status_output_expected_slice_legacy_guard_body_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output expected-slice legacy group child-owner split" => Some(
            "runtime_15_status_output_expected_slice_legacy_group_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output expected-slice guard child-owner split" => Some(
            "runtime_15_status_output_expected_slice_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 expected-slice module-layout guard body folder-backed split" => Some(
            "runtime_15_expected_slice_module_layout_guard_body_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 production guard support row-data child split" => Some(
            "runtime_15_production_guard_support_row_data_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 production guard runtime row-data child split" => Some(
            "runtime_15_production_guard_runtime_row_data_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 production guard status-support priority row-data child split" => Some(
            "runtime_15_production_guard_status_support_priority_row_data_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 production guard status-support priority guard folder-backed split" => Some(
            "runtime_15_production_guard_status_support_priority_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 structure-support expected-slice map child-owner split" => Some(
            "runtime_15_structure_support_expected_slice_map_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 structure-support expected-slice row data folder-backed split" => Some(
            "runtime_15_structure_support_expected_slice_row_data_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 structure-convention warning cleanup" => Some(
            "runtime_15_structure_convention_warning_cleanup_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
