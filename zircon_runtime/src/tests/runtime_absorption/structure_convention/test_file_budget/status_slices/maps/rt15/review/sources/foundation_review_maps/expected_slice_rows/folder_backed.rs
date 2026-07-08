use super::*;

#[test]
fn runtime_15_review_guard_foundation_expected_slice_rows_guard_is_folder_backed() {
    let parent = read_runtime_src(STRUCTURE_GUARD_PATH);
    let children = GUARD_CHILDREN
        .iter()
        .map(|path| read_runtime_src(&format!("tests/runtime_absorption/{path}")))
        .collect::<Vec<_>>()
        .join("\n");

    assert_contains_all(
        "review-foundation expected-slice rows guard route owner",
        &parent,
        &[
            "#[path = \"expected_slice_rows/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"expected_slice_rows/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"expected_slice_rows/paths.rs\"]",
            "mod paths;",
            "#[path = \"expected_slice_rows/route_maps.rs\"]",
            "mod route_maps;",
            "#[path = \"expected_slice_rows/row_data.rs\"]",
            "mod row_data;",
            "#[path = \"expected_slice_rows/status_docs.rs\"]",
            "mod status_docs;",
            "use paths::*;",
        ],
    );
    for moved_anchor in [
        "#[test]",
        ROWS_STATUS_PARENT,
        "read_structure_support_expected_slice_rows()",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "foundation_review_maps/expected_slice_rows.rs should delegate moved anchor {moved_anchor}"
        );
    }
    assert_contains_all(
        "review-foundation expected-slice rows guard children",
        &children,
        &[
            "runtime_15_review_guard_foundation_expected_slice_rows_guard_children_stay_budgeted",
            "runtime_15_review_guard_foundation_expected_slice_rows_are_folder_backed",
            "runtime_15_review_guard_foundation_expected_slice_rows_row_data_is_synced",
            "runtime_15_review_guard_foundation_expected_slice_rows_docs_are_synced",
            GUARD_GUARD,
        ],
    );
}
