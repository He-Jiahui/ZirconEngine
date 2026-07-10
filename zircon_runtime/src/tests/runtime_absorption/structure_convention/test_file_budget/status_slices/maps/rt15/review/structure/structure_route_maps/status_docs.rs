use super::*;

#[test]
fn runtime_15_structure_route_status_date_maps_docs_are_synced() {
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
                ROWS_SLICE,
                ROWS_STATUS,
                "structure_route_maps.rs",
                "structure_route_maps/structure_support_rows.rs",
                "structure_route_maps/guard_rows.rs",
                ROWS_GUARD,
                GUARD_SLICE,
                GUARD_STATUS,
                "structure_route_maps/budgets.rs",
                "structure_route_maps/status_docs.rs",
                GUARD_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "Frameworks 02 structure-route map mirror",
        &frameworks_02,
        &[ROWS_FRAMEWORKS_STATUS, GUARD_FRAMEWORKS_STATUS],
    );
}
