use super::*;

const REVIEW_GUARD_STRUCTURE_ASSERTION_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps/typed_error_structure_maps/structure_assertion_maps.rs";
const REVIEW_GUARD_STRUCTURE_ASSERTION_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps/typed_error_structure_maps/structure_assertion_maps.rs";

pub(super) fn assert_structure_assertions_guard_status_mirrors_are_current() {
    let status_rows = review_guard_status_rows_source();
    let status_map = [
        read_runtime_src(REVIEW_GUARD_STATUS_MAP_PATH),
        read_runtime_src(REVIEW_GUARD_STRUCTURE_ASSERTION_STATUS_MAP_PATH),
    ]
    .join("\n");
    let date_map = [
        read_runtime_src(REVIEW_GUARD_DATE_MAP_PATH),
        read_runtime_src(REVIEW_GUARD_STRUCTURE_ASSERTION_DATE_MAP_PATH),
    ]
    .join("\n");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let frameworks_02 = read_repo(
        "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
    );
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let frameworks_index = read_repo("docs/plans/zircon_runtime/frameworks/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    let common_anchors = &[
        STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_FOLDER_BACKED_SLICE,
        STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_FOLDER_BACKED_STATUS,
        STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD_OWNER,
        STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_SOURCE_TREES_CHILD_OWNER,
        STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_CURRENT_CHECKS_CHILD_OWNER,
        STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_FOLDER_BACKED_CHILD_OWNER,
        STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_BUDGETS_CHILD_OWNER,
        STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_STATUS_MIRRORS_CHILD_OWNER,
        STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_FOLDER_BACKED_GUARD,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("structure guard typed-error row data", status_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("runtime architecture session note", session_note.as_str()),
    ] {
        assert_contains_all(label, source, common_anchors);
    }
    for (label, source) in [
        ("Frameworks 02 plan", frameworks_02.as_str()),
        ("Frameworks index", frameworks_index.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_FOLDER_BACKED_SLICE,
                STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_FOLDER_BACKED_STATUS,
                STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_FRAMEWORKS_STATUS,
                STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_FOLDER_BACKED_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "M3 review status map records typed-error structure assertions guard folder-backed split",
        &status_map,
        &[
            STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_FOLDER_BACKED_SLICE,
            STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_FOLDER_BACKED_STATUS,
        ],
    );
    assert_contains_all(
        "M3 review date map records typed-error structure assertions guard folder-backed split",
        &date_map,
        &[
            STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_FOLDER_BACKED_SLICE,
            "2026-07-07",
        ],
    );
}

#[test]
fn runtime_15_code_review_findings_structure_guard_typed_error_structure_assertions_status_mirrors_are_current(
) {
    assert_structure_assertions_guard_status_mirrors_are_current();
}
