use super::super::*;

const REVIEW_STATUS_SYNC_CHILD_ROWS: &[(&str, &str, &str)] = &[
    (
        "importer_fixture_rows",
        REVIEW_STATUS_SYNC_IMPORTER_FIXTURE_ROWS_PATH,
        "Runtime 15 M3 D13 importer top-row closed status sync",
    ),
    (
        "p0_core_rows",
        REVIEW_STATUS_SYNC_P0_CORE_ROWS_PATH,
        "Runtime 15 M3 P0 F1/F2/F4 top-row closed status sync",
    ),
    (
        "typed_runtime_rows",
        REVIEW_STATUS_SYNC_TYPED_RUNTIME_ROWS_PATH,
        "Runtime 15 M3 F5/F6/F7 typed-error top-row closed status sync",
    ),
    (
        "provider_lookup_rows",
        REVIEW_STATUS_SYNC_PROVIDER_LOOKUP_ROWS_PATH,
        "Runtime 15 M3 F19 scene renderer construction top-row closed status sync",
    ),
    (
        "row_data_owner",
        REVIEW_STATUS_SYNC_ROW_DATA_OWNER_ROWS_PATH,
        REVIEW_STATUS_SYNC_CHILD_SPLIT_STATUS_NAME,
    ),
];

pub(super) fn assert_review_status_sync_parent_delegates_to_children() {
    let review_route = read_runtime_src(REVIEW_STATUS_SYNC_ROW_DATA_PATH);
    assert_contains_all(
        "review status sync route mounts child row groups",
        &review_route,
        &[
            "review_status_sync/importer_fixture_rows.rs",
            "review_status_sync/p0_core_rows.rs",
            "review_status_sync/typed_runtime_rows.rs",
            "review_status_sync/provider_lookup_rows.rs",
            "review_status_sync/row_data_owner.rs",
            "importer_fixture_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "p0_core_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "typed_runtime_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "provider_lookup_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    for moved_row in [
        "Runtime 15 M3 D13 importer top-row closed status sync",
        "Runtime 15 M3 P0 F1/F2/F4 top-row closed status sync",
        "Runtime 15 M3 F5/F6/F7 typed-error top-row closed status sync",
        "Runtime 15 M3 F19 scene renderer construction top-row closed status sync",
        "runtime_15_review_top_row_status_row_data_child_owner_split_static_passed_cargo_deferred",
    ] {
        assert!(
            !review_route.contains(moved_row),
            "review_status_sync.rs should delegate {moved_row} to child row files"
        );
    }
    for (module_name, path, representative_row) in REVIEW_STATUS_SYNC_CHILD_ROWS {
        let child_source = read_runtime_src(path);
        assert_contains_all(
            "review status sync child row file keeps representative row",
            &child_source,
            &[*representative_row],
        );
        assert!(
            review_route.contains(&format!("mod {module_name};")),
            "review_status_sync.rs should mount {module_name}"
        );
        assert!(
            child_source.lines().count() < 100,
            "{path} should stay below its focused review status-sync row-data budget"
        );
    }
}
