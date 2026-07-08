use super::*;

#[test]
fn runtime_15_m3_child_group_status_doc_sources_are_child_owned() {
    let child_groups_guard = read_runtime_src(CHILD_GROUPS_GUARD_PATH);
    let status_docs_source = format!(
        "{}\n{}",
        read_runtime_src(STATUS_DOCS_GUARD_PATH),
        status_docs_child_source_blob()
    );
    let status_row_docs_source = status_row_docs_guard_source();

    for moved_doc_source in [
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
        "docs/plans/engine-code-review-findings-2026-06.md",
        "expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs",
    ] {
        assert!(
            !child_groups_guard.contains(moved_doc_source),
            "runtime_15_m3_child_groups.rs should delegate status-doc source {moved_doc_source}"
        );
        assert!(
            status_docs_source.contains(moved_doc_source),
            "runtime_15_m3_child_group_status_docs children should own status-doc source {moved_doc_source}"
        );
    }

    for delegated_row_doc_source in [
        concat!("lock_", "poison_status.rs"),
        concat!("module_", "convention_status.rs"),
        concat!("review_", "status_sync.rs"),
    ] {
        assert!(
            !status_docs_source.contains(delegated_row_doc_source),
            "runtime_15_m3_child_group_status_docs children should delegate row status-doc source {delegated_row_doc_source}"
        );
        assert!(
            status_row_docs_source.contains(delegated_row_doc_source),
            "runtime_15_m3_child_group_status_row_docs should own row status-doc source {delegated_row_doc_source}"
        );
    }
}
