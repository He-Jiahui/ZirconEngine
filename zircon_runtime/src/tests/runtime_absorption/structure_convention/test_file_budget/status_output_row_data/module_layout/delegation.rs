use super::*;

#[test]
fn runtime_15_status_output_row_data_guard_child_owner_split() {
    let parent = read_runtime_src(STATUS_OUTPUT_ROW_DATA_PARENT_PATH);
    let module_layout_guard = read_runtime_src(MODULE_LAYOUT_PARENT_PATH);

    assert_contains_all(
        "status-output row-data guard parent mounts child owners",
        &parent,
        &[
            "#[path = \"status_output_row_data/module_layout.rs\"]",
            "mod module_layout;",
            "#[path = \"status_output_row_data/module_layout_child_summaries.rs\"]",
            "mod module_layout_child_summaries;",
            "#[path = \"status_output_row_data/module_layout_child_summary_status_docs.rs\"]",
            "mod module_layout_child_summary_status_docs;",
            "#[path = \"status_output_row_data/module_layout_status_docs.rs\"]",
            "mod module_layout_status_docs;",
            "#[path = \"status_output_row_data/evidence_anchors.rs\"]",
            "mod evidence_anchors;",
            "#[path = \"status_output_row_data/runtime_15_row_data.rs\"]",
            "mod runtime_15_row_data;",
            "#[path = \"status_output_row_data/runtime_15_review_guard_row_data.rs\"]",
            "mod runtime_15_review_guard_row_data;",
            "#[path = \"status_output_row_data/runtime_15_review_guard_row_data_moved_rows.rs\"]",
            "mod runtime_15_review_guard_row_data_moved_rows;",
            "#[path = \"status_output_row_data/runtime_15_review_guard_row_data_status_docs.rs\"]",
            "mod runtime_15_review_guard_row_data_status_docs;",
            "#[path = \"status_output_row_data/runtime_15_foundation_row_data.rs\"]",
            "mod runtime_15_foundation_row_data;",
            "#[path = \"status_output_row_data/runtime_15_foundation_row_data_status_docs.rs\"]",
            "mod runtime_15_foundation_row_data_status_docs;",
            "#[path = \"status_output_row_data/runtime_15_m4_row_data.rs\"]",
            "mod runtime_15_m4_row_data;",
            "#[path = \"status_output_row_data/runtime_15_m3_row_data.rs\"]",
            "mod runtime_15_m3_row_data;",
            "#[path = \"status_output_row_data/runtime_15_m3_child_groups.rs\"]",
            "mod runtime_15_m3_child_groups;",
            "#[path = \"status_output_row_data/runtime_15_m3_child_group_moved_rows.rs\"]",
            "mod runtime_15_m3_child_group_moved_rows;",
            "#[path = \"status_output_row_data/runtime_15_m3_child_group_status_docs.rs\"]",
            "mod runtime_15_m3_child_group_status_docs;",
            "#[path = \"status_output_row_data/runtime_15_m3_child_group_status_row_docs.rs\"]",
            "mod runtime_15_m3_child_group_status_row_docs;",
            "#[path = \"status_output_row_data/runtime_15_m2_row_data.rs\"]",
            "mod runtime_15_m2_row_data;",
        ],
    );
    for moved_guard in [
        "fn runtime_15_expected_status_output_rows_accept_variable_evidence_anchors",
        "fn runtime_15_status_output_runtime_15_row_data_is_child_owner",
        "fn runtime_15_status_output_runtime_15_m4_row_data_is_child_owner",
        "fn runtime_15_status_output_m3_review_guard_row_data_is_child_owner",
        "fn runtime_15_status_output_runtime_15_m3_row_data_is_child_owner",
        "fn runtime_15_status_output_m3_row_data_child_owner_split",
        "fn runtime_15_status_output_runtime_15_m2_row_data_is_child_owner",
    ] {
        assert!(
            !parent.contains(moved_guard),
            "status_output_row_data.rs should mount child guard owners instead of defining {moved_guard}"
        );
    }

    assert_contains_all(
        "module-layout parent delegates folder-backed child guards",
        &module_layout_guard,
        &[
            FOLDER_BACKED_STATUS_NAME,
            FOLDER_BACKED_STATUS_ID,
            "mod delegation;",
            "mod child_summaries;",
            "mod status_mirrors;",
            "mod budgets;",
            "module_layout_child_source_blob",
        ],
    );
    for (_, path, guard) in MODULE_LAYOUT_CHILDREN {
        assert_contains_all(
            "module-layout parent records child path and guard",
            &module_layout_guard,
            &[path, guard],
        );
    }
}
