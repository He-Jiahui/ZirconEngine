use super::*;

#[test]
fn runtime_15_review_guard_typed_error_expected_slice_map_rows_are_folder_backed() {
    let status_parent = read_runtime_src(STATUS_REVIEW_TYPED_ERROR_CHILD);
    let date_parent = read_runtime_src(DATE_REVIEW_TYPED_ERROR_CHILD);
    let status_children = read_status_review_typed_error_sources();
    let date_children = read_date_review_typed_error_sources();

    for (label, parent) in [
        ("status typed-error map parent", status_parent.as_str()),
        ("date typed-error map parent", date_parent.as_str()),
    ] {
        assert_contains_all(
            label,
            parent,
            &[
                "#[path = \"typed_error_maps/expected_slice_rows.rs\"]",
                "mod expected_slice_rows;",
                "#[path = \"typed_error_maps/review_guard_rows.rs\"]",
                "mod review_guard_rows;",
                "#[path = \"typed_error_maps/row_data_rows.rs\"]",
                "mod row_data_rows;",
                "#[path = \"typed_error_maps/source_inventory_rows.rs\"]",
                "mod source_inventory_rows;",
                "#[path = \"typed_error_maps/status_doc_rows.rs\"]",
                "mod status_doc_rows;",
            ],
        );
        for moved in [
            "Runtime 15 M3 typed-error convergence guard child-owner split",
            "Runtime 15 M3 review-guard typed-error row-data child split",
            "Runtime 15 M3 typed-error status-doc status mirrors child split",
            "Runtime 15 M3 typed-error source inventory delegation folder-backed ownership child split",
        ] {
            assert!(
                !parent.contains(moved),
                "{label} should delegate typed-error row {moved}"
            );
        }
    }

    assert_contains_all(
        "status typed-error map children",
        &status_children,
        &[
            MAP_ROWS_SLICE,
            MAP_ROWS_STATUS,
            "runtime_15_review_guard_typed_error_expected_slice_map_child_split_static_passed_cargo_deferred",
            "runtime_15_typed_error_convergence_guard_child_owner_split_static_passed_cargo_deferred",
            "runtime_15_review_guard_typed_error_row_data_child_split_static_passed_cargo_deferred",
            "runtime_15_typed_error_status_doc_status_mirrors_child_split_static_passed_cargo_deferred",
            "runtime_15_typed_error_source_inventory_delegation_folder_backed_ownership_child_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "date typed-error map children",
        &date_children,
        &[
            MAP_ROWS_SLICE,
            "Some(\"2026-07-07\")",
            "Some(\"2026-06-25\")",
            "Some(\"2026-07-04\")",
            "Some(\"2026-07-05\")",
        ],
    );
}
