pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 review-guard typed-error structure maps folder-backed split" => Some(
            "runtime_15_review_guard_typed_error_structure_maps_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error structure row-data owner child split" => Some(
            "runtime_15_typed_error_structure_row_data_owner_child_split_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
