pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 review-guard foundation status-date maps folder-backed split" => Some(
            "runtime_15_review_guard_foundation_status_date_maps_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard foundation status-date map guard folder-backed split" => Some(
            "runtime_15_review_guard_foundation_status_date_map_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard foundation route-mount guard folder-backed split" => Some(
            "runtime_15_review_guard_foundation_route_mount_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard foundation status-mirror guard folder-backed split" => Some(
            "runtime_15_review_guard_foundation_status_mirror_guard_folder_backed_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
