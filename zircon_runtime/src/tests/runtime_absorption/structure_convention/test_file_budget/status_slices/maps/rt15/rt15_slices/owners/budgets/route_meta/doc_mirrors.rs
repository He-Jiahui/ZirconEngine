use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_child_owner_budget_route_metadata_docs_are_synced(
) {
    let frameworks_02 = read_repo(
        "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
    );
    for (label, source) in [
        (
            "Runtime 15 plan",
            read_repo(
                "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
            ),
        ),
        (
            "Runtime index",
            read_repo("docs/plans/zircon_runtime/runtime/index.md"),
        ),
        ("Frameworks 02", frameworks_02.clone()),
        (
            "review findings",
            read_repo("docs/plans/engine-code-review-findings-2026-06.md"),
        ),
        (
            "structure convention",
            read_repo("docs/plans/engine-code-structure-convention.md"),
        ),
        (
            "module convention doc",
            read_repo("docs/zircon_runtime/structure/module-convention.md"),
        ),
        (
            "session note",
            read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md"),
        ),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                BUDGET_SLICE,
                BUDGET_STATUS,
                BUDGETS_ROUTE_PATH,
                BUDGETS_SOURCES_PATH,
                BUDGETS_GUARD_BODY_PATH,
                BUDGETS_ROUTE_METADATA_PATH,
                BUDGET_GUARD,
                BUDGET_ROUTE_METADATA_SLICE,
                BUDGET_ROUTE_METADATA_STATUS,
                BUDGET_ROUTE_METADATA_CHILDREN[0],
                BUDGET_ROUTE_METADATA_CHILDREN[1],
                BUDGET_ROUTE_METADATA_CHILDREN[2],
                BUDGET_ROUTE_METADATA_CHILDREN[3],
                BUDGET_ROUTE_METADATA_CHILDREN[4],
                BUDGET_ROUTE_METADATA_CHILDREN[5],
                BUDGET_ROUTE_METADATA_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "Frameworks 02 child-owner budget status mirror",
        &frameworks_02,
        &[
            BUDGET_FRAMEWORKS_STATUS,
            BUDGET_ROUTE_METADATA_FRAMEWORKS_STATUS,
        ],
    );
}
