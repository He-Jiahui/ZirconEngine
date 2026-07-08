pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 typed-error structure assertions guard folder-backed split" => Some(
            "runtime_15_typed_error_structure_assertions_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error structure assertions source reconciliation" => Some(
            "runtime_15_typed_error_structure_assertions_source_reconciliation_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings structure guard typed-error structure assertions folder-backed split" => Some(
            "runtime_15_code_review_findings_structure_guard_typed_error_structure_assertions_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error convergence mounts guard folder-backed split" => Some(
            "runtime_15_typed_error_convergence_mounts_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error convergence mounts root inventory child split" => Some(
            "runtime_15_typed_error_convergence_mounts_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
