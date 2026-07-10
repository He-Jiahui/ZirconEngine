use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_guard_body_status_mirrors_are_registered() {
    let status_rows = read_status_support_expected_slice_rows();
    let status_map = read_status_review_foundation_sources();
    let date_map = read_date_review_foundation_sources();
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let frameworks_02 = read_repo(
        "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
    );
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    assert_contains_all(
        "review guard-body row data",
        &status_rows,
        &[
            REVIEW_ROUTE_GUARD_BODY_SLICE,
            REVIEW_ROUTE_GUARD_BODY_STATUS,
            REVIEW_ROUTE_GUARD_BODY_ROUTE_PATH,
            REVIEW_ROUTE_GUARD_BODY_CHILDREN[0],
            REVIEW_ROUTE_GUARD_BODY_CHILDREN[1],
            REVIEW_ROUTE_GUARD_BODY_CHILDREN[2],
            REVIEW_ROUTE_GUARD_BODY_CHILDREN[3],
            REVIEW_ROUTE_GUARD_BODY_CHILDREN[4],
            REVIEW_ROUTE_GUARD_BODY_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "review guard-body status map",
        &status_map,
        &[
            REVIEW_ROUTE_GUARD_BODY_SLICE,
            REVIEW_ROUTE_GUARD_BODY_STATUS,
        ],
    );
    assert_contains_all(
        "review guard-body date map",
        &date_map,
        &[REVIEW_ROUTE_GUARD_BODY_SLICE, "Some(\"2026-07-06\")"],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("Frameworks 02", frameworks_02.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                REVIEW_ROUTE_GUARD_BODY_SLICE,
                REVIEW_ROUTE_GUARD_BODY_STATUS,
                REVIEW_ROUTE_GUARD_BODY_ROUTE_PATH,
                REVIEW_ROUTE_GUARD_BODY_CHILDREN[0],
                REVIEW_ROUTE_GUARD_BODY_CHILDREN[1],
                REVIEW_ROUTE_GUARD_BODY_CHILDREN[2],
                REVIEW_ROUTE_GUARD_BODY_CHILDREN[3],
                REVIEW_ROUTE_GUARD_BODY_CHILDREN[4],
                REVIEW_ROUTE_GUARD_BODY_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "Frameworks 02 review guard-body status mirror",
        &frameworks_02,
        &[REVIEW_ROUTE_GUARD_BODY_FRAMEWORKS_STATUS],
    );
}
