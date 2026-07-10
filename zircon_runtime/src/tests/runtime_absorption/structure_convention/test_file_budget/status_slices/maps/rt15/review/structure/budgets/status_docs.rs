use super::*;

#[test]
fn runtime_15_structure_support_expected_slice_budgets_status_is_synced() {
    let status_rows = read_structure_support_expected_slice_rows();
    let status_map = read_status_structure_route_map_sources();
    let date_map = read_date_structure_route_map_sources();

    assert_contains_all(
        "structure-support expected-slice budget row data",
        &status_rows,
        &[
            STRUCTURE_SUPPORT_BUDGETS_SLICE,
            STRUCTURE_SUPPORT_BUDGETS_STATUS,
            STRUCTURE_SUPPORT_BUDGETS_ROUTE_PATH,
            STRUCTURE_SUPPORT_BUDGET_CHILDREN[0],
            STRUCTURE_SUPPORT_BUDGET_CHILDREN[4],
            STRUCTURE_SUPPORT_BUDGETS_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "structure-support expected-slice budget status/date maps",
        &format!("{status_map}\n{date_map}"),
        &[
            STRUCTURE_SUPPORT_BUDGETS_SLICE,
            STRUCTURE_SUPPORT_BUDGETS_STATUS,
            "2026-07-06",
        ],
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
                STRUCTURE_SUPPORT_BUDGETS_SLICE,
                STRUCTURE_SUPPORT_BUDGETS_STATUS,
                "structure/budgets/status_docs.rs",
                STRUCTURE_SUPPORT_BUDGETS_GUARD,
            ],
        );
    }
    assert_contains_all(
        "Frameworks 02 structure-support budget mirror",
        &read_repo(
            "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
        ),
        &[
            STRUCTURE_SUPPORT_BUDGETS_SLICE,
            STRUCTURE_SUPPORT_BUDGETS_STATUS,
            STRUCTURE_SUPPORT_BUDGETS_FRAMEWORKS_STATUS,
        ],
    );
}
