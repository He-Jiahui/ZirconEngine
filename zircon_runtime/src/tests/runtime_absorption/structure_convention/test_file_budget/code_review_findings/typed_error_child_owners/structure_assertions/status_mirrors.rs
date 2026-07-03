use super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_typed_error_structure_assertions_guard_folder_backed_status_is_current() {
    let status_rows = read_runtime_src(REVIEW_GUARD_STATUS_ROWS_PATH);
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
        ("typed-error structure row data", status_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("runtime architecture session note", session_note.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                TYPED_ERROR_STRUCTURE_ASSERTIONS_GUARD_SPLIT_NAME,
                TYPED_ERROR_STRUCTURE_ASSERTIONS_GUARD_SPLIT_ID,
                TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD,
                TYPED_ERROR_STRUCTURE_CONVERGENCE_MOUNTS_CHILD,
                TYPED_ERROR_STRUCTURE_DELEGATION_CHILD,
                TYPED_ERROR_STRUCTURE_CHILD_OWNERSHIP_CHILD,
                TYPED_ERROR_STRUCTURE_STATUS_MIRRORS_CHILD,
                TYPED_ERROR_NATIVE_STRUCTURE_CHILD,
                TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD,
                "runtime_15_typed_error_structure_assertions_are_child_owner",
                "runtime_15_typed_error_structure_assertions_children_are_child_owned",
                "runtime_15_typed_error_native_plugin_loader_structure_is_child_owner",
                "runtime_15_typed_error_structure_moved_guard_absence_is_child_owner",
                "runtime_15_typed_error_structure_assertions_guard_folder_backed_status_is_current",
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "M3 review status map records typed-error structure assertions folder-backed split",
        &status_map,
        &[
            TYPED_ERROR_STRUCTURE_ASSERTIONS_GUARD_SPLIT_NAME,
            TYPED_ERROR_STRUCTURE_ASSERTIONS_GUARD_SPLIT_ID,
        ],
    );
    assert_contains_all(
        "M3 review date map records typed-error structure assertions folder-backed split",
        &date_map,
        &[
            TYPED_ERROR_STRUCTURE_ASSERTIONS_GUARD_SPLIT_NAME,
            "2026-07-02",
        ],
    );

    let parent = read_runtime_src(TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD);
    for (path, source) in [(TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD, parent)]
        .into_iter()
        .chain(structure_assertion_guard_child_sources())
    {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
