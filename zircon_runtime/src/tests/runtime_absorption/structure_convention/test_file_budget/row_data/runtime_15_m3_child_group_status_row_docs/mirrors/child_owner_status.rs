use super::*;

const CHILD_OWNER_STATUS_ROW_NAME: &str =
    "Runtime 15 M3 child-group status-row-doc guard child-owner split";
const CHILD_OWNER_STATUS_ROW_ID: &str =
    "runtime_15_m3_child_group_status_row_docs_child_owner_split_static_passed_cargo_deferred";

#[test]
fn runtime_15_m3_child_group_status_row_doc_child_owner_status_is_current() {
    let status_rows = read_runtime_src(STATUS_SUPPORT_STATUS_DOCS_ROWS_PATH);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    let child_owner_status_anchors = [
        CHILD_OWNER_STATUS_ROW_NAME,
        CHILD_OWNER_STATUS_ROW_ID,
        "structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_status_docs.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_status_row_docs.rs",
        CHILD_OWNER_GUARD_NAME,
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        (
            "status-output Runtime 15 M3 production support status-doc rows",
            status_rows.as_str(),
        ),
    ] {
        assert_contains_all(label, source, &child_owner_status_anchors);
    }
}
