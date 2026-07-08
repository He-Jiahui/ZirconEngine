pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 code review findings test folder split" => {
            Some("runtime_15_code_review_findings_tests_folder_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code-review standalone harness current-path sync" => {
            Some("runtime_15_code_review_standalone_harness_current_path_sync_static_passed_cargo_deferred")
        }
        _ => None,
    }
}
