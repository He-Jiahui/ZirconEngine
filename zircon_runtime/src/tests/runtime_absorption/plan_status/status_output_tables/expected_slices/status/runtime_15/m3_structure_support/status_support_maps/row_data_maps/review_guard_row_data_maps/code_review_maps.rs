pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 review-guard code-review row-data guard folder-backed split" => Some(
            "runtime_15_review_guard_code_review_rows_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard code-review row-data root inventory child split" => Some(
            "runtime_15_review_guard_code_review_rows_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard code-review status-mirror child split" => Some(
            "runtime_15_review_guard_code_review_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard code-review export/status source reconciliation" => Some(
            "runtime_15_review_guard_code_review_export_status_source_reconciliation_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard typed-error structure-assertions row-data folder-backed split" => Some(
            "runtime_15_review_guard_typed_error_structure_assertions_row_data_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard typed-error structure-assertions guard folder-backed split" => Some(
            "runtime_15_review_guard_typed_error_structure_assertions_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error structure row-data guard folder-backed split" => Some(
            "runtime_15_typed_error_structure_rows_guard_folder_backed_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
