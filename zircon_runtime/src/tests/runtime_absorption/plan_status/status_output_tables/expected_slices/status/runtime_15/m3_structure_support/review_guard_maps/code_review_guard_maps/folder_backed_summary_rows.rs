pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 code review findings folder-backed summary child-owner split" => Some(
            "runtime_15_code_review_findings_folder_backed_summary_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings folder-backed summary guard folder-backed split" => Some(
            "runtime_15_code_review_findings_folder_backed_summary_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings folder-backed summary child-ownership guard folder-backed split" => {
            Some("runtime_15_code_review_findings_folder_backed_summary_child_ownership_guard_folder_backed_static_passed_cargo_deferred")
        }
        _ => None,
    }
}
