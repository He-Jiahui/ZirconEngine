use super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_code_review_findings_source_inventory_status_mirrors_are_child_owner() {
    let delegation = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/source_inventory/delegation.rs",
    );
    let status_mirrors = read_runtime_src(SOURCE_INVENTORY_STATUS_MIRRORS_CHILD);

    for moved_anchor in [
        "let status_rows = source_inventory_status_rows_source();",
        "let runtime_15_plan =",
        "review-guard status map",
        "review-guard date map",
    ] {
        assert!(
            !delegation.contains(moved_anchor),
            "source inventory delegation child should not own status mirror anchor `{moved_anchor}`"
        );
        assert!(
            status_mirrors.contains(moved_anchor),
            "source inventory status mirrors child should own status mirror anchor `{moved_anchor}`"
        );
    }

    assert_source_inventory_status_mirrors_are_current();
}

fn assert_source_inventory_status_mirrors_are_current() {
    let status_rows = source_inventory_status_rows_source();
    let status_map = read_runtime_src(REVIEW_GUARD_STATUS_MAP_PATH);
    let date_map = read_runtime_src(REVIEW_GUARD_DATE_MAP_PATH);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    for (label, source) in [
        ("structure guard row data", status_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("runtime implementation session", session_note.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                SOURCE_INVENTORY_FOLDER_BACKED_SPLIT_NAME,
                SOURCE_INVENTORY_FOLDER_BACKED_SPLIT_ID,
                SOURCE_INVENTORY_STATUS_MIRROR_CHILD_OWNER_SPLIT_NAME,
                SOURCE_INVENTORY_STATUS_MIRROR_CHILD_OWNER_SPLIT_ID,
                SOURCE_INVENTORY_CHILD,
                "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/source_inventory/model.rs",
                "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/source_inventory/reads.rs",
                "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/source_inventory/budgets.rs",
                "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/source_inventory/delegation.rs",
                SOURCE_INVENTORY_STATUS_MIRRORS_CHILD,
                "runtime_15_code_review_findings_source_inventory_is_child_owner",
                SOURCE_INVENTORY_STATUS_MIRROR_CHILD_OWNER_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "review-guard status map",
        &status_map,
        &[
            SOURCE_INVENTORY_FOLDER_BACKED_SPLIT_NAME,
            SOURCE_INVENTORY_FOLDER_BACKED_SPLIT_ID,
            SOURCE_INVENTORY_STATUS_MIRROR_CHILD_OWNER_SPLIT_NAME,
            SOURCE_INVENTORY_STATUS_MIRROR_CHILD_OWNER_SPLIT_ID,
        ],
    );
    assert_contains_all(
        "review-guard date map",
        &date_map,
        &[
            SOURCE_INVENTORY_FOLDER_BACKED_SPLIT_NAME,
            "2026-07-02",
            SOURCE_INVENTORY_STATUS_MIRROR_CHILD_OWNER_SPLIT_NAME,
            SOURCE_INVENTORY_STATUS_MIRROR_CHILD_OWNER_DATE,
        ],
    );
}
