use super::*;

#[test]
fn runtime_15_review_guard_foundation_expected_slice_rows_docs_are_synced() {
    let frameworks_02 = read_repo(
        "docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md",
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
                "foundation_review_maps/expected_slice_rows.rs",
                "foundation_review_maps/expected_slice_rows/route_metadata_rows.rs",
                "foundation_review_maps/expected_slice_rows/root_route_rows.rs",
                "foundation_review_maps/expected_slice_rows/expected_slice_map_rows.rs",
                ROWS_GUARD,
                GUARD_SLICE,
                GUARD_STATUS,
                "foundation_review_maps/expected_slice_rows/budgets.rs",
                "foundation_review_maps/expected_slice_rows/status_docs.rs",
                GUARD_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "Frameworks 02 review-foundation expected-slice rows mirror",
        &frameworks_02,
        &[ROWS_FRAMEWORKS_STATUS, GUARD_FRAMEWORKS_STATUS],
    );
}
