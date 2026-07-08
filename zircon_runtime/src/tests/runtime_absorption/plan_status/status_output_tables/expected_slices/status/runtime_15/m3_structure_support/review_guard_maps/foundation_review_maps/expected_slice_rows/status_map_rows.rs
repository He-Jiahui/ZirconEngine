pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 review-guard source status maps folder-backed split" => Some(
            "runtime_15_review_guard_source_status_maps_folder_backed_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
