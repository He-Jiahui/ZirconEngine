use super::*;

#[test]
fn runtime_15_status_output_foundation_row_data_status_docs_are_child_owner() {
    let status_output_row_data_parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data.rs",
    );
    let foundation_row_data_guard = read_runtime_src(FOUNDATION_ROW_DATA_GUARD_PATH);
    let status_docs_guard = read_runtime_src(STATUS_DOCS_PARENT_PATH);
    let child_sources = status_doc_child_source_blob();

    assert_contains_all(
        "status-output row-data guard mounts foundation status-doc child",
        &status_output_row_data_parent,
        &[
            "#[path = \"status_output_row_data/runtime_15_foundation_row_data_status_docs.rs\"]",
            "mod runtime_15_foundation_row_data_status_docs;",
        ],
    );
    for moved_doc_source in [
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
        "docs/plans/engine-code-review-findings-2026-06.md",
    ] {
        assert!(
            !foundation_row_data_guard.contains(moved_doc_source),
            "runtime_15_foundation_row_data.rs should delegate status-doc source {moved_doc_source}"
        );
        assert!(
            child_sources.contains(moved_doc_source),
            "runtime_15_foundation_row_data_status_docs folder should own status-doc source {moved_doc_source}"
        );
    }
    for (moved_doc_source, child_owned_anchor) in [
        (
            "expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs",
            "STATUS_SUPPORT_STATUS_MAP_PATH",
        ),
        (
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs",
            "PRODUCTION_GUARD_SUPPORT_PATH",
        ),
    ] {
        assert!(
            !foundation_row_data_guard.contains(moved_doc_source),
            "runtime_15_foundation_row_data.rs should delegate status-doc source {moved_doc_source}"
        );
        assert!(
            status_docs_guard.contains(moved_doc_source),
            "runtime_15_foundation_row_data_status_docs parent should inventory status-doc source {moved_doc_source}"
        );
        assert!(
            child_sources.contains(child_owned_anchor),
            "runtime_15_foundation_row_data_status_docs folder should own status-doc source anchor {child_owned_anchor}"
        );
    }
    assert_contains_all(
        "foundation row-data status-doc parent records split anchors",
        &status_docs_guard,
        STATUS_DOC_STATUS_ANCHORS,
    );
    assert_contains_all(
        "foundation row-data status-doc parent mounts folder-backed children",
        &status_docs_guard,
        &[
            "mod delegation;",
            "mod doc_mirrors;",
            "mod row_count;",
            "mod status_maps;",
        ],
    );

    for (_, child_path, guard_name) in STATUS_DOC_CHILDREN {
        assert!(
            status_docs_guard.contains(child_path),
            "foundation row-data status-doc parent should mount child path {child_path}"
        );
        assert!(
            child_sources.contains(guard_name),
            "foundation row-data status-doc child {child_path} should define {guard_name}"
        );
    }
}
