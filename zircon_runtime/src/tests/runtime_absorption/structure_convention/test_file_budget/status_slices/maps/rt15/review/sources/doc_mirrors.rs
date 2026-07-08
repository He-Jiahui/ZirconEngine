use super::*;

#[test]
fn runtime_15_review_guard_source_inventory_docs_are_synced() {
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
                SOURCES_SLICE,
                SOURCES_STATUS,
                SOURCES_ROUTE_PATH,
                SOURCES_CHILDREN[0],
                SOURCES_CHILDREN[1],
                SOURCES_CHILDREN[2],
                SOURCES_CHILDREN[3],
                SOURCES_CHILDREN[4],
                SOURCES_CHILDREN[5],
                SOURCES_CHILDREN[6],
                SOURCES_CHILDREN[7],
                SOURCES_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "Frameworks 02 review-guard source inventory status mirror",
        &frameworks_02,
        &[SOURCES_FRAMEWORKS_STATUS],
    );
}
