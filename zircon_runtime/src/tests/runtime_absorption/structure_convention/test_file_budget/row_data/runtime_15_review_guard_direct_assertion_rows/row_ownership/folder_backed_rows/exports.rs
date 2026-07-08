use super::*;

#[test]
fn runtime_15_review_guard_direct_assertion_row_data_exports_are_child_owned() {
    let direct_assertion_rows = read_runtime_src(DIRECT_ASSERTION_ROWS_PATH);
    let code_review_rows = read_runtime_src(CODE_REVIEW_ROWS_PATH);
    let review_guard_splits = read_runtime_src(REVIEW_GUARD_SPLITS_PATH);
    let runtime_15_m3_rows = read_runtime_src(RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15_rows = read_runtime_src(RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH);
    let top_level_rows = read_runtime_src(TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH);

    for (module_name, child_path, representative_row) in DIRECT_ASSERTION_ROW_DATA_CHILD_ROWS {
        let path_attr = format!("#[path = \"direct_assertion_rows/{module_name}.rs\"]");
        let module_mount = format!("mod {module_name};");
        assert_contains_all(
            "direct assertion row-data parent mounts child",
            &direct_assertion_rows,
            &[path_attr.as_str(), module_mount.as_str()],
        );
        let child = read_runtime_src(child_path);
        assert_contains_all(child_path, &child, &[*representative_row]);
        assert!(
            !code_review_rows.contains(representative_row),
            "code_review_rows.rs should not own direct-assertion child literal {representative_row}"
        );
    }

    assert_contains_all(
        "code-review row-data parent exports direct-assertion groups",
        &code_review_rows,
        &[
            "DIRECT_ASSERTION_F12_EXPECTED_STATUS_OUTPUT_SLICES",
            "DIRECT_ASSERTION_ROOT_PARENT_EXPECTED_STATUS_OUTPUT_SLICES",
            "DIRECT_ASSERTION_RENDER_EXPECTED_STATUS_OUTPUT_SLICES",
            "DIRECT_ASSERTION_F8_EXPECTED_STATUS_OUTPUT_SLICES",
            "DIRECT_ASSERTION_P0_EXPECTED_STATUS_OUTPUT_SLICES",
            "DIRECT_ASSERTION_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "review-guard split parent exports direct-assertion child groups",
        &review_guard_splits,
        &[
            "CODE_REVIEW_DIRECT_ASSERTION_F12_EXPECTED_STATUS_OUTPUT_SLICES",
            "CODE_REVIEW_DIRECT_ASSERTION_ROOT_PARENT_EXPECTED_STATUS_OUTPUT_SLICES",
            "CODE_REVIEW_DIRECT_ASSERTION_RENDER_EXPECTED_STATUS_OUTPUT_SLICES",
            "CODE_REVIEW_DIRECT_ASSERTION_F8_EXPECTED_STATUS_OUTPUT_SLICES",
            "CODE_REVIEW_DIRECT_ASSERTION_P0_EXPECTED_STATUS_OUTPUT_SLICES",
            "CODE_REVIEW_DIRECT_ASSERTION_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    for source in [
        runtime_15_m3_rows.as_str(),
        runtime_15_rows.as_str(),
        top_level_rows.as_str(),
    ] {
        assert_contains_all(
            "Runtime 15 expected-status aggregation exports direct-assertion child groups",
            source,
            &[
                "REVIEW_GUARD_CODE_REVIEW_DIRECT_ASSERTION_F12_EXPECTED_STATUS_OUTPUT_SLICES",
                "REVIEW_GUARD_CODE_REVIEW_DIRECT_ASSERTION_ROOT_PARENT_EXPECTED_STATUS_OUTPUT_SLICES",
                "REVIEW_GUARD_CODE_REVIEW_DIRECT_ASSERTION_RENDER_EXPECTED_STATUS_OUTPUT_SLICES",
                "REVIEW_GUARD_CODE_REVIEW_DIRECT_ASSERTION_F8_EXPECTED_STATUS_OUTPUT_SLICES",
                "REVIEW_GUARD_CODE_REVIEW_DIRECT_ASSERTION_P0_EXPECTED_STATUS_OUTPUT_SLICES",
                "REVIEW_GUARD_CODE_REVIEW_DIRECT_ASSERTION_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
            ],
        );
    }
}
