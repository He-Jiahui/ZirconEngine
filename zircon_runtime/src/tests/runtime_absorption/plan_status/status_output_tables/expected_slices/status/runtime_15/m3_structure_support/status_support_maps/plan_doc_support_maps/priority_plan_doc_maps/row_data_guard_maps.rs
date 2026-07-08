pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 priority plan docs row-data owner child split" => Some(
            "runtime_15_priority_plan_docs_row_data_owner_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs owner-guard row-data child split" => Some(
            "runtime_15_priority_plan_docs_owner_guard_row_data_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 priority plan docs row-data guard folder-backed split" => Some(
            "runtime_15_priority_plan_docs_row_data_guard_folder_backed_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
