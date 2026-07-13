use super::*;

const CHILD_SPLIT_STATUS_MIRROR_NAME: &str =
    "Runtime 15 M3 child-group status-row-doc status-mirror child split";
const CHILD_SPLIT_STATUS_MIRROR_ID: &str = "runtime_15_m3_child_group_status_row_docs_status_mirror_child_split_static_passed_cargo_deferred";

#[test]
fn runtime_15_m3_child_group_status_row_doc_status_mirror_status_rows_are_current() {
    let status_rows = read_runtime_src(STATUS_SUPPORT_STATUS_DOCS_ROWS_PATH);
    let status_map = read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH);
    let date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    let status_anchors = [
        CHILD_SPLIT_STATUS_MIRROR_NAME,
        CHILD_SPLIT_STATUS_MIRROR_ID,
        "structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_status_row_docs/status_mirrors.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_status_row_docs/mirrors/child_split_status.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_status_row_docs/mirrors/child_owner_status.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_status_row_docs/mirrors/m3_row_status.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_status_row_docs/mirrors/folder_backed_status.rs",
        "runtime_15_m3_child_group_status_row_doc_status_mirror_children_are_child_owned",
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        (
            "production support status-doc rows record child-group status-row-doc status-mirror child split",
            status_rows.as_str(),
        ),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "Runtime 15 status-support map records child-group status-row-doc status-mirror child split",
        &status_map,
        &[STATUS_MIRROR_CHILD_SPLIT_NAME, STATUS_MIRROR_CHILD_SPLIT_ID],
    );
    assert_contains_all(
        "Runtime 15 date map records child-group status-row-doc status-mirror child split",
        &date_map,
        &[STATUS_MIRROR_CHILD_SPLIT_NAME, "2026-07-04"],
    );
}
