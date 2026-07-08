use super::*;

#[test]
fn runtime_15_naming_boundary_expected_slice_guard_body_route_metadata_docs_are_synced() {
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
                ROUTE_METADATA_SLICE,
                ROUTE_METADATA_STATUS,
                GUARD_BODY_ROUTE_CHILDREN[2],
                ROUTE_METADATA_CHILDREN[0],
                ROUTE_METADATA_CHILDREN[1],
                ROUTE_METADATA_CHILDREN[2],
                ROUTE_METADATA_CHILDREN[3],
                ROUTE_METADATA_CHILDREN[4],
                ROUTE_METADATA_CHILDREN[5],
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
