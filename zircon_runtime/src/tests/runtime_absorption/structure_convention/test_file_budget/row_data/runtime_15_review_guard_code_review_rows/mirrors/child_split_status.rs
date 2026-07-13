use super::*;

const STATUS_MIRROR_CHILD_SPLIT_STATUS_NAME: &str =
    "Runtime 15 M3 review-guard code-review status-mirror child split";
const STATUS_MIRROR_CHILD_SPLIT_STATUS_ID: &str =
    "runtime_15_review_guard_code_review_status_mirror_child_split_static_passed_cargo_deferred";
const STATUS_MIRROR_CHILD_SPLIT_GUARD_NAME: &str =
    "runtime_15_review_guard_code_review_status_mirror_children_are_folder_backed";

#[test]
fn runtime_15_review_guard_code_review_status_mirror_status_rows_are_current() {
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_support_rows = status_support_review_guard_source_blob();
    let status_support_status_map = status_support_status_map_source_blob();
    let status_support_date_map = status_support_date_map_source_blob();

    let status_split_anchors = [
        STATUS_MIRROR_CHILD_SPLIT_STATUS_NAME,
        STATUS_MIRROR_CHILD_SPLIT_STATUS_ID,
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/status_mirrors.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/mirrors/child_split_status.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/mirrors/code_review_owner.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/mirrors/structure_guard_rows.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/mirrors/folder_backed.rs",
        STATUS_MIRROR_CHILD_SPLIT_GUARD_NAME,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        (
            "production guard support rows",
            status_support_rows.as_str(),
        ),
    ] {
        assert_contains_all(label, source, &status_split_anchors);
    }
    assert_contains_all(
        "Runtime 15 status-support expected status map records status-mirror child split",
        &status_support_status_map,
        &[
            STATUS_MIRROR_CHILD_SPLIT_STATUS_NAME,
            STATUS_MIRROR_CHILD_SPLIT_STATUS_ID,
        ],
    );
    assert_contains_all(
        "Runtime 15 status-support expected date map records status-mirror child split",
        &status_support_date_map,
        &[STATUS_MIRROR_CHILD_SPLIT_STATUS_NAME, "2026-07-04"],
    );
}
