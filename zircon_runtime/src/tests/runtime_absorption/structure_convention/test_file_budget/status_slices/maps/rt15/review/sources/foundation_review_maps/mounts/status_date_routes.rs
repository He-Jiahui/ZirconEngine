use super::*;

#[test]
fn runtime_15_review_guard_foundation_status_date_maps_are_folder_backed() {
    let status_parent = read_runtime_src(STATUS_REVIEW_FOUNDATION_CHILD);
    let date_parent = read_runtime_src(DATE_REVIEW_FOUNDATION_CHILD);
    let status_children = read_status_review_foundation_sources();
    let date_children = read_date_review_foundation_sources();

    assert_contains_all(
        "review foundation status/date parents route child maps",
        &format!("{status_parent}\n{date_parent}"),
        &[
            "#[path = \"foundation_review_maps/code_review_rows.rs\"]",
            "#[path = \"foundation_review_maps/expected_slice_rows.rs\"]",
            "#[path = \"foundation_review_maps/f8_rows.rs\"]",
            "#[path = \"foundation_review_maps/late_api_rows.rs\"]",
            "#[path = \"foundation_review_maps/p0_rows.rs\"]",
            "expected_slice_rows::expected_status_for_slice(slice)",
            "code_review_rows::expected_status_for_slice(slice)",
            "p0_rows::expected_status_for_slice(slice)",
            "f8_rows::expected_status_for_slice(slice)",
            "late_api_rows::expected_status_for_slice(slice)",
            "expected_slice_rows::expected_date_for_slice(slice)",
            "code_review_rows::expected_date_for_slice(slice)",
            "p0_rows::expected_date_for_slice(slice)",
            "f8_rows::expected_date_for_slice(slice)",
            "late_api_rows::expected_date_for_slice(slice)",
        ],
    );

    for moved_literal in [
        "Runtime 15 M3 review-guard structure row data folder-backed split",
        "Runtime 15 M3 P0 robustness review guard child-owner split",
        "Runtime 15 M3 F8 API convergence review guard child-owner split",
        "Runtime 15 M3 late API cleanup review guard child-owner split",
    ] {
        assert!(
            !status_parent.contains(moved_literal),
            "status foundation_review_maps.rs should delegate {moved_literal}"
        );
        assert!(
            !date_parent.contains(moved_literal),
            "date foundation_review_maps.rs should delegate {moved_literal}"
        );
    }

    assert_contains_all(
        "review foundation child maps own moved rows",
        &format!("{status_children}\n{date_children}"),
        &[
            REVIEW_GUARD_STRUCTURE_ROW_DATA_SLICE,
            REVIEW_GUARD_STRUCTURE_ROW_DATA_STATUS,
            REVIEW_FOUNDATION_MAPS_SLICE,
            REVIEW_FOUNDATION_MAPS_STATUS,
            "Runtime 15 M3 P0 robustness review guard child-owner split",
            "runtime_15_f8_api_convergence_review_guard_child_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 late API cleanup review guard child-owner split",
            "Some(\"2026-07-06\")",
        ],
    );
}
