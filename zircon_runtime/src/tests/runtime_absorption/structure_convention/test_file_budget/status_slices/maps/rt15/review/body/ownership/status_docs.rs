use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_root_guard_body_child_ownership_status_is_mirrored() {
    let status_rows = read_review_guard_structure_rows();
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

    assert_contains_all(
        "review guard child ownership row data",
        &status_rows,
        &[
            CHILD_OWNERSHIP_SLICE,
            CHILD_OWNERSHIP_STATUS,
            CHILD_OWNERSHIP_ROUTE_PATH,
            CHILD_OWNERSHIP_CHILDREN[0],
            CHILD_OWNERSHIP_CHILDREN[1],
            CHILD_OWNERSHIP_CHILDREN[2],
            CHILD_OWNERSHIP_CHILDREN[3],
            CHILD_OWNERSHIP_CHILDREN[4],
            CHILD_OWNERSHIP_CHILDREN[5],
            CHILD_OWNERSHIP_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "status review-guard child ownership map",
        &status_map,
        &[CHILD_OWNERSHIP_SLICE, CHILD_OWNERSHIP_STATUS],
    );
    assert_contains_all(
        "date review-guard child ownership map",
        &date_map,
        &[CHILD_OWNERSHIP_SLICE, "Some(\"2026-07-06\")"],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("Frameworks 02", frameworks_02.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                CHILD_OWNERSHIP_SLICE,
                CHILD_OWNERSHIP_STATUS,
                CHILD_OWNERSHIP_FRAMEWORKS_STATUS,
                CHILD_OWNERSHIP_ROUTE_PATH,
                CHILD_OWNERSHIP_CHILDREN[0],
                CHILD_OWNERSHIP_CHILDREN[1],
                CHILD_OWNERSHIP_CHILDREN[2],
                CHILD_OWNERSHIP_CHILDREN[3],
                CHILD_OWNERSHIP_CHILDREN[4],
                CHILD_OWNERSHIP_CHILDREN[5],
                CHILD_OWNERSHIP_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
}
