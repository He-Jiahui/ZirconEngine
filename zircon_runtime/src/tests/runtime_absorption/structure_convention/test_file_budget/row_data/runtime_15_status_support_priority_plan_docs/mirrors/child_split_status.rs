use super::*;

#[test]
fn runtime_15_priority_plan_docs_status_mirror_children_are_child_owned() {
    let status_rows = production_guard_support_priority_rows_source_blob();
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/priority_plan_doc_maps/status_mirror_maps.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/priority_plan_doc_maps/status_mirror_maps.rs",
    );

    let status_anchors = [
        STATUS_MIRROR_CHILD_SPLIT_NAME,
        STATUS_MIRROR_CHILD_SPLIT_ID,
        "structure_convention/test_file_budget/row_data/runtime_15_status_support_priority_plan_docs/status_mirrors.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_status_support_priority_plan_docs/mirrors/child_split_status.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_status_support_priority_plan_docs/mirrors/row_owner_status.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_status_support_priority_plan_docs/mirrors/folder_backed_status.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_status_support_priority_plan_docs/mirrors/owner_guard_status.rs",
        "runtime_15_priority_plan_docs_status_mirror_children_are_child_owned",
        "Cargo gate deferred",
    ];
    assert_contains_all(
        "production support row data records priority-plan-doc status-mirror child split",
        &status_rows,
        &status_anchors,
    );
    assert_contains_all(
        "Runtime 15 status-support expected status map records priority-plan-doc status-mirror child split",
        &status_map,
        &[STATUS_MIRROR_CHILD_SPLIT_NAME, STATUS_MIRROR_CHILD_SPLIT_ID],
    );
    assert_contains_all(
        "Runtime 15 status-support expected date map records priority-plan-doc status-mirror child split",
        &date_map,
        &[STATUS_MIRROR_CHILD_SPLIT_NAME, "2026-07-04"],
    );

    let doc_anchors = [
        STATUS_MIRROR_CHILD_SPLIT_NAME,
        STATUS_MIRROR_CHILD_SPLIT_ID,
        "runtime_15_priority_plan_docs_status_mirror_children_are_child_owned",
    ];
    for (label, path) in [
        (
            "Runtime 15 plan",
            "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
        ),
        (
            "Runtime index",
            "docs/plans/zircon_runtime/runtime/index.md",
        ),
        (
            "structure convention",
            "docs/plans/engine-code-structure-convention.md",
        ),
        (
            "review findings",
            "docs/plans/engine-code-review-findings-2026-06.md",
        ),
        (
            "module convention doc",
            "docs/zircon_runtime/structure/module-convention.md",
        ),
        (
            "runtime implementation session",
            ".codex/sessions/20260612-0847-runtime-architecture-implementation.md",
        ),
    ] {
        let source = read_repo(path);
        assert_contains_all(label, &source, &doc_anchors);
    }
}
