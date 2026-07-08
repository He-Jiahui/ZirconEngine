pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 code review findings typed-error structure guard child-owner split" => Some(
            "runtime_15_code_review_findings_typed_error_structure_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error structure assertions guard child-owner split" => Some(
            "runtime_15_typed_error_structure_assertions_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error structure source/status-map sync" => Some(
            "runtime_15_typed_error_structure_source_status_map_sync_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
