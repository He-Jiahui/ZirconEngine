use super::*;

#[test]
fn runtime_15_structure_support_expected_slice_status_docs_are_synced() {
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
                STRUCTURE_SUPPORT_GUARD_SLICE,
                STRUCTURE_SUPPORT_GUARD_STATUS,
                STRUCTURE_SUPPORT_GUARD_NAME,
                "structure/status_mirrors.rs",
            ],
        );
        assert_contains_all(
            label,
            &source,
            &[
                STRUCTURE_SUPPORT_STATUS_MIRRORS_SLICE,
                STRUCTURE_SUPPORT_STATUS_MIRRORS_STATUS,
                STRUCTURE_SUPPORT_STATUS_MIRRORS_ROUTE_PATH,
                STRUCTURE_SUPPORT_STATUS_MIRROR_CHILDREN[0],
                STRUCTURE_SUPPORT_STATUS_MIRROR_CHILDREN[1],
                STRUCTURE_SUPPORT_STATUS_MIRROR_CHILDREN[2],
                STRUCTURE_SUPPORT_STATUS_MIRROR_CHILDREN[3],
                STRUCTURE_SUPPORT_STATUS_MIRRORS_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "Frameworks 02 structure-support guard mirror",
        &read_repo(
            "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
        ),
        &[
            STRUCTURE_SUPPORT_GUARD_SLICE,
            STRUCTURE_SUPPORT_GUARD_STATUS,
            STRUCTURE_SUPPORT_GUARD_FRAMEWORKS_STATUS,
        ],
    );
    assert_contains_all(
        "Frameworks 02 structure-support status mirrors mirror",
        &read_repo(
            "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
        ),
        &[
            STRUCTURE_SUPPORT_STATUS_MIRRORS_SLICE,
            STRUCTURE_SUPPORT_STATUS_MIRRORS_STATUS,
            STRUCTURE_SUPPORT_STATUS_MIRRORS_FRAMEWORKS_STATUS,
        ],
    );
}
