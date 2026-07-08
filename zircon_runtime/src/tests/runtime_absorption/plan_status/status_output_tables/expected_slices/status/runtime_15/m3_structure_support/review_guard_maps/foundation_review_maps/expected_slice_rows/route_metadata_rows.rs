pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 review-guard expected-slice maps folder-backed split" => Some(
            "runtime_15_review_guard_expected_slice_maps_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard expected-slice route metadata child split" => Some(
            "runtime_15_review_guard_expected_slice_route_metadata_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard expected-slice route metadata guard folder-backed split" => Some(
            "runtime_15_review_guard_expected_slice_route_metadata_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard expected-slice route metadata budgets folder-backed split" => Some(
            "runtime_15_review_guard_expected_slice_route_metadata_budgets_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard expected-slice route metadata route mounts folder-backed split" => Some(
            "runtime_15_review_guard_expected_slice_route_metadata_route_mounts_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard expected-slice route metadata route mounts folder-backed guard body split" => Some(
            "runtime_15_review_guard_expected_slice_route_metadata_route_mounts_folder_backed_guard_body_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard expected-slice route metadata status mirrors folder-backed split" => Some(
            "runtime_15_review_guard_expected_slice_route_metadata_status_mirrors_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard expected-slice route child sources folder-backed split" => Some(
            "runtime_15_review_guard_expected_slice_route_child_sources_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard expected-slice route metadata source constants folder-backed split" => Some(
            "runtime_15_review_guard_expected_slice_route_metadata_source_constants_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard source structure paths folder-backed split" => Some(
            "runtime_15_review_guard_source_structure_paths_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard expected-slice guard body folder-backed split" => Some(
            "runtime_15_review_guard_expected_slice_guard_body_folder_backed_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
