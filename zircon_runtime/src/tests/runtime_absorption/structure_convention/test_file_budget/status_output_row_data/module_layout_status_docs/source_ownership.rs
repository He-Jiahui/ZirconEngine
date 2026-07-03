use super::*;

#[test]
fn runtime_15_module_layout_status_doc_sources_are_child_owned() {
    let module_layout_guard = read_runtime_src(MODULE_LAYOUT_GUARD_PATH);
    let child_sources = module_layout_status_doc_child_source_blob();

    for moved_doc_source in [
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs",
        "expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs",
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
        "docs/plans/engine-code-review-findings-2026-06.md",
        "docs/zircon_runtime/structure/module-convention.md",
    ] {
        assert!(
            !module_layout_guard.contains(moved_doc_source),
            "module_layout.rs should delegate status-doc source {moved_doc_source}"
        );
        assert!(
            child_sources.contains(moved_doc_source),
            "module-layout status-doc child should own status-doc source {moved_doc_source}"
        );
    }

    for (path, source) in [
        (
            "runtime_15_foundation_row_data.rs",
            read_runtime_src(FOUNDATION_ROW_DATA_GUARD_PATH),
        ),
        (
            "runtime_15_review_guard_row_data.rs",
            read_runtime_src(REVIEW_GUARD_ROW_DATA_GUARD_PATH),
        ),
        (
            "runtime_15_m3_child_groups.rs",
            read_runtime_src(M3_CHILD_GROUPS_GUARD_PATH),
        ),
    ] {
        assert!(
            !source.contains(
                "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
            ),
            "{path} should delegate Runtime 15 plan doc anchors to its status-doc child owner"
        );
    }
}
