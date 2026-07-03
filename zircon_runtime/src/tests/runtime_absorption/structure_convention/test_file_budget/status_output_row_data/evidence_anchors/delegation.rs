use super::*;

#[test]
fn runtime_15_status_output_evidence_anchors_guard_is_folder_backed() {
    let status_output_row_data_parent = read_runtime_src(STATUS_OUTPUT_ROW_DATA_PARENT_PATH);
    let guard_parent = read_runtime_src(EVIDENCE_ANCHORS_GUARD_PATH);
    let child_sources = evidence_anchors_guard_child_source_blob();

    assert_contains_all(
        "status-output row-data guard mounts evidence anchors guard",
        &status_output_row_data_parent,
        &[
            "#[path = \"status_output_row_data/evidence_anchors.rs\"]",
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
            guard_parent.contains(child_path),
            "evidence anchors guard should mount child path {child_path}"
        );
        assert!(
            child_sources.contains(guard_name),
            "evidence anchors guard child {child_path} should define {guard_name}"
        );
    }
}
