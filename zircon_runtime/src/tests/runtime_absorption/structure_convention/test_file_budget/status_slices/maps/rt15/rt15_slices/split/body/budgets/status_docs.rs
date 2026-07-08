use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_maps_guard_body_budgets_status_is_synced() {
    let row_data = read_top_level_support_row_sources();
    let status_map = read_status_support_status_map_sources();
    let date_map = read_status_support_date_map_sources();

    assert_contains_all(
        "Runtime 15 expected-slice maps guard-body budget row data",
        &row_data,
        &[
            BUDGETS_SLICE,
            BUDGETS_STATUS,
            BUDGETS_ROUTE_PATH,
            GUARD_BODY_BUDGET_CHILDREN[0],
            GUARD_BODY_BUDGET_CHILDREN[5],
            BUDGETS_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "Runtime 15 expected-slice maps guard-body budget status/date maps",
        &format!("{status_map}\n{date_map}"),
        &[BUDGETS_SLICE, BUDGETS_STATUS, "2026-07-06"],
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
                BUDGETS_SLICE,
                BUDGETS_STATUS,
                "split/body/budgets/status_docs.rs",
                BUDGETS_GUARD,
            ],
        );
    }
    assert_contains_all(
        "Frameworks 02 expected-slice maps guard-body budget mirror",
        &read_repo(
            "docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md",
        ),
        &[BUDGETS_SLICE, BUDGETS_STATUS, BUDGETS_FRAMEWORKS_STATUS],
    );
}
