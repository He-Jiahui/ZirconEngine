use super::*;

#[test]
fn runtime_15_naming_boundary_expected_slice_guard_body_route_metadata_docs_are_synced() {
    let frameworks_02 = read_repo(
        "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
    );
    for (label, source) in [
        (
            "Runtime 15 plan",
            read_repo(
                "docs/plans/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
            ),
        ),
        (
            "Runtime index",
            read_repo("docs/plans/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md"),
        ),
        ("Frameworks 02", frameworks_02.clone()),
        (
            "review findings",
            read_repo("docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"),
        ),
        (
            "structure convention",
            read_repo("docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md"),
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
                ROUTE_METADATA_SLICE,
                ROUTE_METADATA_STATUS,
                ROUTE_METADATA_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "Frameworks 02 naming-boundary route metadata mirror",
        &frameworks_02,
        &[
            GUARD_BODY_ROUTE_FRAMEWORKS_STATUS,
            ROUTE_METADATA_FRAMEWORKS_STATUS,
        ],
    );
}
