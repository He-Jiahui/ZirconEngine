use super::*;

pub(super) fn assert_status_support_review_rows_route_children_are_current() {
    let parent = read_runtime_src(REVIEW_GUARD_STATUS_SUPPORT_ROWS_PATH);
    let status_support_rows = read_runtime_src(REVIEW_GUARD_STATUS_SUPPORT_ROWS_PARENT_PATH);
    for (module, export) in REVIEW_GUARD_STATUS_SUPPORT_ROW_GROUPS {
        let mount = format!("#[path = \"review_guard_rows/{module}.rs\"]");
        let export = format!("review_guard_rows::{export}");
        assert_contains_all(
            "status-support review row parent mounts child rows",
            &parent,
            &[mount.as_str()],
        );
        assert_contains_all(
            "status-support row parent exports review row groups",
            &status_support_rows,
            &[export.as_str()],
        );
    }
    assert!(
        !parent.contains("Runtime 15 M3 review-guard status-support row-data folder-backed split"),
        "status-support review_guard_rows.rs should route row groups instead of owning status row tuples",
    );

    assert_contains_all(
        "status-support review row children own representative rows",
        &review_guard_status_support_review_rows_source_blob(),
        &[
            "Runtime 15 M3 review guard status row-data child-owner split",
            "Runtime 15 M3 review-guard status-support row-data folder-backed split",
            "Runtime 15 M3 review-guard typed-error rows guard folder-backed split",
            "Runtime 15 M3 review-guard row-data budgets guard folder-backed split",
            STATUS_SUPPORT_REVIEW_ROWS_ROW_DATA_STATUS_ID,
            STATUS_SUPPORT_REVIEW_ROWS_ROW_DATA_GUARD_NAME,
        ],
    );
}
