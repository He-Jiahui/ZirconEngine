use super::*;

#[test]
fn runtime_15_review_guard_code_review_expected_slice_map_rows_guard_is_folder_backed() {
    let parent = read_runtime_src(STRUCTURE_GUARD_PATH);
    let children = GUARD_CHILDREN
        .iter()
        .map(|path| read_runtime_src(&format!("tests/runtime_absorption/{path}")))
        .collect::<Vec<_>>()
        .join("\n");

    assert_contains_all(
        "code-review map rows guard route owner",
        &parent,
        &[
            "#[path = \"code_review_map_rows/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"code_review_map_rows/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"code_review_map_rows/paths.rs\"]",
            "mod paths;",
            "#[path = \"code_review_map_rows/route_maps.rs\"]",
            "mod route_maps;",
            "#[path = \"code_review_map_rows/row_data.rs\"]",
            "mod row_data;",
            "#[path = \"code_review_map_rows/status_docs.rs\"]",
            "mod status_docs;",
            "use paths::*;",
        ],
    );
    for moved_anchor in [
        "#[test]",
        MAP_ROWS_STATUS,
        "read_status_review_code_review_sources()",
        "read_structure_support_expected_slice_rows()",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "code_review_map_rows.rs should delegate moved anchor {moved_anchor}"
        );
    }
    assert_contains_all(
        "code-review map rows guard children",
        &children,
        &[
            "runtime_15_review_guard_code_review_expected_slice_map_rows_guard_children_stay_budgeted",
            "runtime_15_review_guard_code_review_expected_slice_map_rows_guard_is_folder_backed",
            "runtime_15_review_guard_code_review_expected_slice_map_rows_are_folder_backed",
            "runtime_15_review_guard_code_review_expected_slice_map_rows_row_data_is_synced",
            "runtime_15_review_guard_code_review_expected_slice_map_rows_docs_are_synced",
            GUARD_GUARD,
        ],
    );
}
