use super::*;

const MOVED_STATUS_DOC_SOURCES: &[&str] = &[
    "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs",
    "expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs",
    "expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs",
    "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
    "docs/plans/zircon_runtime/runtime/index.md",
    "docs/plans/engine-code-structure-convention.md",
    "docs/plans/engine-code-review-findings-2026-06.md",
    "docs/zircon_runtime/structure/module-convention.md",
];

#[test]
fn runtime_15_module_layout_child_summary_status_doc_sources_are_child_owned() {
    let child_summary_guard = read_runtime_src(CHILD_SUMMARY_GUARD_PATH);
    let status_doc_child_sources = child_summary_status_doc_child_source_blob();

    for moved_doc_source in MOVED_STATUS_DOC_SOURCES {
        assert!(
            !child_summary_guard.contains(moved_doc_source),
            "module_layout_child_summaries.rs should delegate status-doc source {moved_doc_source}"
        );
        assert!(
            status_doc_child_sources.contains(moved_doc_source),
            "module_layout_child_summary_status_docs children should own status-doc source {moved_doc_source}"
        );
    }
}
