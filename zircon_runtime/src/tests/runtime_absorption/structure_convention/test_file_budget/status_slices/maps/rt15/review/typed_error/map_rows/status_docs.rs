use super::*;

#[test]
fn runtime_15_review_guard_typed_error_expected_slice_map_rows_docs_are_synced() {
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
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                MAP_ROWS_SLICE,
                MAP_ROWS_STATUS,
                "review/typed_error_maps.rs",
                "review/typed_error_maps/expected_slice_rows.rs",
                MAP_ROWS_GUARD,
                GUARD_SLICE,
                GUARD_STATUS,
                "typed_error/map_rows.rs",
                "typed_error/map_rows/budgets.rs",
                "typed_error/map_rows/status_docs.rs",
                GUARD_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "Frameworks 02 typed-error map rows mirrors",
        &frameworks_02,
        &[MAP_ROWS_FRAMEWORKS_STATUS, GUARD_FRAMEWORKS_STATUS],
    );
}
