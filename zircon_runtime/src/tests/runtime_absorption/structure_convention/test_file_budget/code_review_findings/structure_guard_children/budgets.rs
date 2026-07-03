use super::super::super::*;
use super::*;

pub(super) fn assert_structure_guard_children_line_budgets() {
    let sources = [
        (
            STRUCTURE_GUARD_CHILD_OWNER,
            read_runtime_src(STRUCTURE_GUARD_CHILD_OWNER),
        ),
        (F8_CHILD_OWNER, read_runtime_src(F8_CHILD_OWNER)),
        (
            F8_DELEGATION_CHILD_OWNER,
            read_runtime_src(F8_DELEGATION_CHILD_OWNER),
        ),
        (
            F8_ROUTE_OWNERSHIP_CHILD_OWNER,
            read_runtime_src(F8_ROUTE_OWNERSHIP_CHILD_OWNER),
        ),
        (
            F8_STATUS_MIRRORS_CHILD_OWNER,
            read_runtime_src(F8_STATUS_MIRRORS_CHILD_OWNER),
        ),
        (
            F8_BUDGETS_CHILD_OWNER,
            read_runtime_src(F8_BUDGETS_CHILD_OWNER),
        ),
        (
            LATE_API_CLEANUP_CHILD_OWNER,
            read_runtime_src(LATE_API_CLEANUP_CHILD_OWNER),
        ),
        (
            LATE_API_CLEANUP_DELEGATION_CHILD_OWNER,
            read_runtime_src(LATE_API_CLEANUP_DELEGATION_CHILD_OWNER),
        ),
        (
            LATE_API_CLEANUP_ROUTE_OWNERSHIP_CHILD_OWNER,
            read_runtime_src(LATE_API_CLEANUP_ROUTE_OWNERSHIP_CHILD_OWNER),
        ),
        (
            LATE_API_CLEANUP_STATUS_MIRRORS_CHILD_OWNER,
            read_runtime_src(LATE_API_CLEANUP_STATUS_MIRRORS_CHILD_OWNER),
        ),
        (
            LATE_API_CLEANUP_BUDGETS_CHILD_OWNER,
            read_runtime_src(LATE_API_CLEANUP_BUDGETS_CHILD_OWNER),
        ),
        (P0_CHILD_OWNER, read_runtime_src(P0_CHILD_OWNER)),
        (
            P0_DELEGATION_CHILD_OWNER,
            read_runtime_src(P0_DELEGATION_CHILD_OWNER),
        ),
        (
            P0_ROUTE_OWNERSHIP_CHILD_OWNER,
            read_runtime_src(P0_ROUTE_OWNERSHIP_CHILD_OWNER),
        ),
        (
            P0_STATUS_MIRRORS_CHILD_OWNER,
            read_runtime_src(P0_STATUS_MIRRORS_CHILD_OWNER),
        ),
        (
            P0_BUDGETS_CHILD_OWNER,
            read_runtime_src(P0_BUDGETS_CHILD_OWNER),
        ),
        (
            P0_NATIVE_FIXTURE_LEAF_OWNER,
            read_runtime_src(P0_NATIVE_FIXTURE_LEAF_OWNER),
        ),
        (
            P0_NATIVE_FIXTURE_DELEGATION_CHILD_OWNER,
            read_runtime_src(P0_NATIVE_FIXTURE_DELEGATION_CHILD_OWNER),
        ),
        (
            P0_NATIVE_FIXTURE_LEAF_OWNERSHIP_CHILD_OWNER,
            read_runtime_src(P0_NATIVE_FIXTURE_LEAF_OWNERSHIP_CHILD_OWNER),
        ),
        (
            P0_NATIVE_FIXTURE_STATUS_MIRRORS_CHILD_OWNER,
            read_runtime_src(P0_NATIVE_FIXTURE_STATUS_MIRRORS_CHILD_OWNER),
        ),
        (
            P0_NATIVE_FIXTURE_BUDGETS_CHILD_OWNER,
            read_runtime_src(P0_NATIVE_FIXTURE_BUDGETS_CHILD_OWNER),
        ),
        (
            PLUGIN_IMPORTER_DX_CHILD_OWNER,
            read_runtime_src(PLUGIN_IMPORTER_DX_CHILD_OWNER),
        ),
        (
            PLUGIN_IMPORTER_DX_TOP_LEVEL_DELEGATION_CHILD_OWNER,
            read_runtime_src(PLUGIN_IMPORTER_DX_TOP_LEVEL_DELEGATION_CHILD_OWNER),
        ),
        (
            PLUGIN_IMPORTER_DX_TOP_LEVEL_CHILD_OWNERSHIP_CHILD_OWNER,
            read_runtime_src(PLUGIN_IMPORTER_DX_TOP_LEVEL_CHILD_OWNERSHIP_CHILD_OWNER),
        ),
        (
            PLUGIN_IMPORTER_DX_TOP_LEVEL_STATUS_MIRRORS_CHILD_OWNER,
            read_runtime_src(PLUGIN_IMPORTER_DX_TOP_LEVEL_STATUS_MIRRORS_CHILD_OWNER),
        ),
        (
            PLUGIN_IMPORTER_DX_TOP_LEVEL_BUDGETS_CHILD_OWNER,
            read_runtime_src(PLUGIN_IMPORTER_DX_TOP_LEVEL_BUDGETS_CHILD_OWNER),
        ),
        (
            PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_CHILD_OWNER,
            read_runtime_src(PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_CHILD_OWNER),
        ),
        (
            PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD_OWNER,
            read_runtime_src(PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD_OWNER),
        ),
        (
            PLUGIN_IMPORTER_DX_STATUS_DOCS_CHILD_OWNER,
            read_runtime_src(PLUGIN_IMPORTER_DX_STATUS_DOCS_CHILD_OWNER),
        ),
        (
            TYPED_ERROR_CHILD_OWNER,
            read_runtime_src(TYPED_ERROR_CHILD_OWNER),
        ),
        (
            TYPED_ERROR_TOP_LEVEL_DELEGATION_CHILD_OWNER,
            read_runtime_src(TYPED_ERROR_TOP_LEVEL_DELEGATION_CHILD_OWNER),
        ),
        (
            TYPED_ERROR_TOP_LEVEL_CHILD_OWNERSHIP_CHILD_OWNER,
            read_runtime_src(TYPED_ERROR_TOP_LEVEL_CHILD_OWNERSHIP_CHILD_OWNER),
        ),
        (
            TYPED_ERROR_TOP_LEVEL_STATUS_MIRRORS_CHILD_OWNER,
            read_runtime_src(TYPED_ERROR_TOP_LEVEL_STATUS_MIRRORS_CHILD_OWNER),
        ),
        (
            TYPED_ERROR_TOP_LEVEL_BUDGETS_CHILD_OWNER,
            read_runtime_src(TYPED_ERROR_TOP_LEVEL_BUDGETS_CHILD_OWNER),
        ),
        (
            TYPED_ERROR_SOURCE_INVENTORY_CHILD_OWNER,
            read_runtime_src(TYPED_ERROR_SOURCE_INVENTORY_CHILD_OWNER),
        ),
        (
            TYPED_ERROR_STATUS_DOCS_CHILD_OWNER,
            read_runtime_src(TYPED_ERROR_STATUS_DOCS_CHILD_OWNER),
        ),
        (
            TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD_OWNER,
            read_runtime_src(TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD_OWNER),
        ),
        (
            TYPED_ERROR_STRUCTURE_CONVERGENCE_MOUNTS_CHILD_OWNER,
            read_runtime_src(TYPED_ERROR_STRUCTURE_CONVERGENCE_MOUNTS_CHILD_OWNER),
        ),
        (
            TYPED_ERROR_STRUCTURE_DELEGATION_CHILD_OWNER,
            read_runtime_src(TYPED_ERROR_STRUCTURE_DELEGATION_CHILD_OWNER),
        ),
        (
            TYPED_ERROR_STRUCTURE_CHILD_OWNERSHIP_CHILD_OWNER,
            read_runtime_src(TYPED_ERROR_STRUCTURE_CHILD_OWNERSHIP_CHILD_OWNER),
        ),
        (
            TYPED_ERROR_STRUCTURE_STATUS_MIRRORS_CHILD_OWNER,
            read_runtime_src(TYPED_ERROR_STRUCTURE_STATUS_MIRRORS_CHILD_OWNER),
        ),
        (
            TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD_OWNER,
            read_runtime_src(TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD_OWNER),
        ),
        (
            TYPED_ERROR_NATIVE_STRUCTURE_CHILD_OWNER,
            read_runtime_src(TYPED_ERROR_NATIVE_STRUCTURE_CHILD_OWNER),
        ),
        (
            STATUS_DOCS_CHILD_OWNER,
            read_runtime_src(STATUS_DOCS_CHILD_OWNER),
        ),
        (
            STATUS_DOCS_SOURCE_ANCHORS_CHILD_OWNER,
            read_runtime_src(STATUS_DOCS_SOURCE_ANCHORS_CHILD_OWNER),
        ),
        (
            STATUS_DOCS_STATUS_ANCHORS_CHILD_OWNER,
            read_runtime_src(STATUS_DOCS_STATUS_ANCHORS_CHILD_OWNER),
        ),
    ];
    for (path, source) in sources {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
    for (path, source) in structure_guard_child_sources() {
        let line_count = source.lines().count();
        assert!(
            line_count < 400,
            "{path} should stay below the focused structure-guard child budget; got {line_count} lines"
        );
    }
}

#[test]
fn runtime_15_code_review_findings_structure_guard_children_folder_backed_status_is_current() {
    let status_map = read_runtime_src(REVIEW_GUARD_STATUS_MAP_PATH);
    let date_map = read_runtime_src(REVIEW_GUARD_DATE_MAP_PATH);
    let structure_guard_rows = review_guard_status_rows_source();
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    assert_structure_guard_children_line_budgets();
    assert_contains_all(
        "review-guard status map",
        &status_map,
        &[
            STRUCTURE_GUARD_FOLDER_BACKED_SPLIT_NAME,
            STRUCTURE_GUARD_FOLDER_BACKED_SPLIT_ID,
        ],
    );
    for (label, source) in [
        ("structure guard row data", structure_guard_rows.as_str()),
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
                STRUCTURE_GUARD_FOLDER_BACKED_SPLIT_NAME,
                STRUCTURE_GUARD_FOLDER_BACKED_SPLIT_ID,
                "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children.rs",
                "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/delegation.rs",
                "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/review_guard_groups.rs",
                "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/plugin_importer.rs",
                "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/status_docs.rs",
                "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/budgets.rs",
                "runtime_15_code_review_findings_structure_guard_children_are_mounted",
            ],
        );
    }
    assert_contains_all(
        "review-guard date map",
        &date_map,
        &[STRUCTURE_GUARD_FOLDER_BACKED_SPLIT_NAME, "2026-07-02"],
    );
}
