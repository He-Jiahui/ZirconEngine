use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_route_metadata_docs_are_synced(
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
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                ROUTE_SLICE,
                ROUTE_STATUS,
                SPLIT_LAYOUT_PATH,
                SPLIT_LAYOUT_SOURCES_PATH,
                SPLIT_LAYOUT_GUARD_BODY_PATH,
                SPLIT_LAYOUT_ROUTE_METADATA_PATH,
                SPLIT_LAYOUT_STATUS_MIRRORS_PATH,
                ROUTE_GUARD,
                ROUTE_METADATA_SLICE,
                ROUTE_METADATA_STATUS,
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
        "Frameworks 02 child-owner split-layout status mirror",
        &frameworks_02,
        &[ROUTE_FRAMEWORKS_STATUS, ROUTE_METADATA_FRAMEWORKS_STATUS],
    );
}
