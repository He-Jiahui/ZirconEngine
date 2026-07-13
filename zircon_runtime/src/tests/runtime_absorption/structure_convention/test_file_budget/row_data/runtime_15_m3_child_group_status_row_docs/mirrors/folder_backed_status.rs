use super::*;

const FOLDER_BACKED_STATUS_ROW_NAME: &str =
    "Runtime 15 M3 child-group status-row-doc guard folder-backed split";
const FOLDER_BACKED_STATUS_ROW_ID: &str =
    "runtime_15_m3_child_group_status_row_docs_guard_folder_backed_static_passed_cargo_deferred";

#[test]
fn runtime_15_m3_child_group_status_row_doc_folder_backed_status_is_current() {
    let status_rows = read_runtime_src(STATUS_SUPPORT_STATUS_DOCS_ROWS_PATH);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    let folder_backed_status_anchors = [
        FOLDER_BACKED_STATUS_ROW_NAME,
        FOLDER_BACKED_STATUS_ROW_ID,
        "structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_status_row_docs.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_status_row_docs/delegation.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_status_row_docs/row_sources.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_status_row_docs/status_maps.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_status_row_docs/status_mirrors.rs",
        FOLDER_BACKED_GUARD_NAME,
        CHILD_OWNER_GUARD_NAME,
        "Cargo gate deferred",
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
        assert_contains_all(label, source, &folder_backed_status_anchors);
    }
}
