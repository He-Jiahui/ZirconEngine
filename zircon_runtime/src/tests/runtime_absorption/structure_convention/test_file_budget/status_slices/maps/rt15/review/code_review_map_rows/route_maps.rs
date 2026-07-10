use super::*;

#[test]
fn runtime_15_review_guard_code_review_expected_slice_map_rows_are_folder_backed() {
    let status_parent = read_runtime_src(STATUS_REVIEW_CODE_REVIEW_CHILD);
    let date_parent = read_runtime_src(DATE_REVIEW_CODE_REVIEW_CHILD);
    let status_children = read_status_review_code_review_sources();
    let date_children = read_date_review_code_review_sources();

    for (label, parent) in [
        ("status code-review map parent", status_parent.as_str()),
        ("date code-review map parent", date_parent.as_str()),
    ] {
        assert_contains_all(
            label,
            parent,
            &[
                "#[path = \"code_review_guard_maps/direct_assertion_rows.rs\"]",
                "mod direct_assertion_rows;",
                "#[path = \"code_review_guard_maps/expected_slice_rows.rs\"]",
                "mod expected_slice_rows;",
                "#[path = \"code_review_guard_maps/folder_backed_summary_rows.rs\"]",
                "mod folder_backed_summary_rows;",
                "#[path = \"code_review_guard_maps/source_inventory_rows.rs\"]",
                "mod source_inventory_rows;",
                "#[path = \"code_review_guard_maps/status_doc_rows.rs\"]",
                "mod status_doc_rows;",
                "#[path = \"code_review_guard_maps/structure_guard_rows.rs\"]",
                "mod structure_guard_rows;",
            ],
        );
        for moved in [
            "Runtime 15 M3 code review findings structure guard child-owner split",
            "Runtime 15 M3 code review findings status-doc guard child-owner split",
            "Runtime 15 M3 code review findings folder-backed summary child-owner split",
            "Runtime 15 M3 code review findings source inventory child-owner split",
            "Runtime 15 M3 code review findings direct assertions child-owner split",
        ] {
            assert!(
                !parent.contains(moved),
                "{label} should delegate code-review row {moved}"
            );
        }
    }

    assert_contains_all(
        "status code-review map children",
        &status_children,
        &[
            MAP_ROWS_SLICE,
            MAP_ROWS_STATUS,
            "runtime_15_code_review_findings_structure_guard_child_owner_split_static_passed_cargo_deferred",
            "runtime_15_code_review_findings_status_docs_child_owner_split_static_passed_cargo_deferred",
            "runtime_15_code_review_findings_folder_backed_summary_child_owner_split_static_passed_cargo_deferred",
            "runtime_15_code_review_findings_source_inventory_child_owner_split_static_passed_cargo_deferred",
            "runtime_15_code_review_findings_direct_assertions_child_owner_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "date code-review map children",
        &date_children,
        &[
            MAP_ROWS_SLICE,
            "Some(\"2026-07-07\")",
            "Some(\"2026-06-29\")",
            "Some(\"2026-06-30\")",
            "Some(\"2026-07-04\")",
        ],
    );
}
