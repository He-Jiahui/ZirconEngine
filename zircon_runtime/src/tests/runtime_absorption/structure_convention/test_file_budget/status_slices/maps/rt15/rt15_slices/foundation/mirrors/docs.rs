use super::*;

#[test]
fn runtime_15_foundation_expected_slice_maps_docs_are_synced() {
    let frameworks_02 = read_repo(
        "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
    );
    let frameworks_index = read_repo("docs/plans/zircon_runtime/frameworks/index.md");
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
        ("Frameworks index", frameworks_index.clone()),
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
                FOUNDATION_MAP_SLICE,
                FOUNDATION_MAP_STATUS,
                "runtime_15/foundation.rs",
                "runtime_15/foundation/lock_poison.rs",
                FOUNDATION_MAP_GUARD,
                FOUNDATION_GUARD_SLICE,
                FOUNDATION_GUARD_STATUS,
                FOUNDATION_GUARD_FRAMEWORKS_STATUS,
                "rt15_slices/foundation.rs",
                "rt15_slices/foundation/status_mirrors.rs",
                FOUNDATION_GUARD,
                FOUNDATION_STATUS_MIRRORS_SLICE,
                FOUNDATION_STATUS_MIRRORS_STATUS,
                FOUNDATION_STATUS_MIRRORS_FRAMEWORKS_STATUS,
                "rt15_slices/foundation/mirrors/row_data.rs",
                FOUNDATION_STATUS_MIRRORS_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "Frameworks 02 foundation expected-slice mirrors",
        &format!("{frameworks_02}\n{frameworks_index}"),
        &[
            FOUNDATION_MAP_FRAMEWORKS_STATUS,
            FOUNDATION_GUARD_FRAMEWORKS_STATUS,
            FOUNDATION_STATUS_MIRRORS_FRAMEWORKS_STATUS,
        ],
    );
}
