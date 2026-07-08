use super::*;

#[test]
fn runtime_15_status_output_evidence_anchors_guard_is_folder_backed() {
    let status_output_row_data_parent = read_runtime_src(STATUS_OUTPUT_ROW_DATA_PARENT_PATH);
    let guard_parent = read_runtime_src(EVIDENCE_ANCHORS_GUARD_PATH);
    let root_statuses = read_runtime_src(ROOT_STATUSES_PATH);
    let root_child_rows = read_runtime_src(ROOT_CHILD_ROWS_PATH);
    let child_sources = evidence_anchors_guard_child_source_blob();

    assert_contains_all(
        "status-output row-data guard mounts evidence anchors guard",
        &status_output_row_data_parent,
        &[
            "#[path = \"row_data/evidence_anchors.rs\"]",
            "mod evidence_anchors;",
        ],
    );
    assert_contains_all(
        "evidence anchors guard mounts folder-backed children",
        &guard_parent,
        &[
            "mod budgets;",
            "mod delegation;",
            "mod status_mirrors;",
            "mod variable_evidence;",
            "mod root_child_rows;",
            "mod root_inventory;",
            "mod root_owner_paths;",
            "mod root_paths;",
            "mod root_statuses;",
            "pub(super) use root_child_rows::*;",
            "pub(super) use root_owner_paths::*;",
            "pub(super) use root_paths::*;",
            "pub(super) use root_statuses::*;",
        ],
    );
    assert_contains_all(
        "Runtime 15 evidence anchors root statuses preserve historical anchors",
        &root_statuses,
        &[
            VARIABLE_EVIDENCE_STATUS_NAME,
            VARIABLE_EVIDENCE_STATUS_ID,
            VARIABLE_EVIDENCE_GUARD_NAME,
            FOLDER_BACKED_STATUS_NAME,
            FOLDER_BACKED_STATUS_ID,
            FOLDER_BACKED_GUARD_NAME,
        ],
    );
    for (_, child_path, guard_name) in EVIDENCE_ANCHORS_GUARD_CHILDREN {
        assert!(
            root_child_rows.contains(child_path),
            "evidence anchors root child inventory should mount child path {child_path}"
        );
        assert!(
            child_sources.contains(guard_name),
            "evidence anchors guard child {child_path} should define {guard_name}"
        );
    }
}
