use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_maps_are_folder_backed() {
    let status_review_parent = read_runtime_src(STATUS_REVIEW_CHILD);
    let date_review_parent = read_runtime_src(DATE_REVIEW_CHILD);
    let status_foundation = read_status_review_foundation_sources();
    let status_code_review = read_status_review_code_review_sources();
    let status_typed_error_structure = read_status_review_typed_error_structure_sources();
    let status_plugin_importer = read_runtime_src(STATUS_REVIEW_PLUGIN_IMPORTER_CHILD);
    let status_top_row = read_runtime_src(STATUS_REVIEW_TOP_ROW_CHILD);
    let date_foundation = read_date_review_foundation_sources();
    let date_code_review = read_date_review_code_review_sources();
    let date_typed_error_structure = read_date_review_typed_error_structure_sources();
    let date_plugin_importer = read_runtime_src(DATE_REVIEW_PLUGIN_IMPORTER_CHILD);
    let date_top_row = read_runtime_src(DATE_REVIEW_TOP_ROW_CHILD);

    assert_contains_all(
        "review expected-slice status/date parents mount route children",
        &format!("{status_review_parent}\n{date_review_parent}"),
        &[
            "#[path = \"review_guard_maps/foundation_review_maps.rs\"]",
            "#[path = \"review_guard_maps/code_review_guard_maps.rs\"]",
            "#[path = \"review_guard_maps/typed_error_structure_maps.rs\"]",
            "#[path = \"review_guard_maps/plugin_importer_maps.rs\"]",
            "#[path = \"review_guard_maps/top_row_review_maps.rs\"]",
            "foundation_review_maps::expected_status_for_slice(slice)",
            "code_review_guard_maps::expected_status_for_slice(slice)",
            "typed_error_structure_maps::expected_status_for_slice(slice)",
            "plugin_importer_maps::expected_status_for_slice(slice)",
            "top_row_review_maps::expected_status_for_slice(slice)",
            "foundation_review_maps::expected_date_for_slice(slice)",
            "code_review_guard_maps::expected_date_for_slice(slice)",
            "typed_error_structure_maps::expected_date_for_slice(slice)",
            "plugin_importer_maps::expected_date_for_slice(slice)",
            "top_row_review_maps::expected_date_for_slice(slice)",
        ],
    );

    for moved_literal in [
        "Runtime 15 M3 P0 robustness review guard child-owner split",
        "Runtime 15 M3 code review findings status-doc guard folder-backed split",
        "Runtime 15 M3 typed-error native plugin loader routes child split",
        "Runtime 15 M3 plugin-importer D13 SDK parent-mount guard child split",
        "Runtime 15 M3 D12 runtime helper export macro review sync",
    ] {
        assert!(
            !status_review_parent.contains(moved_literal),
            "status review_guard_maps.rs should delegate {moved_literal}"
        );
        assert!(
            !date_review_parent.contains(moved_literal),
            "date review_guard_maps.rs should delegate {moved_literal}"
        );
    }

    assert_contains_all(
        "review expected-slice route children own representative literals",
        &format!(
            "{status_foundation}\n{status_code_review}\n{status_typed_error_structure}\n{status_plugin_importer}\n{status_top_row}\n{date_foundation}\n{date_code_review}\n{date_typed_error_structure}\n{date_plugin_importer}\n{date_top_row}"
        ),
        &[
            "Runtime 15 M3 review-guard expected-slice maps folder-backed split",
            "runtime_15_review_guard_expected_slice_maps_folder_backed_static_passed_cargo_deferred",
            "Runtime 15 M3 P0 robustness review guard child-owner split",
            "Runtime 15 M3 code review findings status-doc guard folder-backed split",
            "Runtime 15 M3 typed-error native plugin loader routes child split",
            "Runtime 15 M3 plugin-importer D13 SDK parent-mount guard child split",
            "Runtime 15 M3 D12 runtime helper export macro review sync",
            "Some(\"2026-07-05\")",
        ],
    );
}
