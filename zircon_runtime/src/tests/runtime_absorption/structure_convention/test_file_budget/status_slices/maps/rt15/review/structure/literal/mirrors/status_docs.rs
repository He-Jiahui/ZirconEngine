use super::*;

#[test]
fn runtime_15_structure_support_expected_slice_literal_ownership_status_docs_are_synced() {
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
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                LITERAL_OWNERSHIP_SLICE,
                LITERAL_OWNERSHIP_STATUS,
                LITERAL_OWNERSHIP_GUARD,
                "structure/literal/status_mirrors.rs",
            ],
        );
        assert_contains_all(
            label,
            &source,
            &[
                LITERAL_STATUS_MIRRORS_SLICE,
                LITERAL_STATUS_MIRRORS_STATUS,
                LITERAL_STATUS_MIRRORS_ROUTE_PATH,
                LITERAL_STATUS_MIRROR_CHILDREN[0],
                LITERAL_STATUS_MIRROR_CHILDREN[1],
                LITERAL_STATUS_MIRROR_CHILDREN[2],
                LITERAL_STATUS_MIRROR_CHILDREN[3],
                LITERAL_STATUS_MIRRORS_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "Frameworks 02 literal ownership mirrors",
        &read_repo(
            "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
        ),
        &[
            LITERAL_OWNERSHIP_SLICE,
            LITERAL_OWNERSHIP_STATUS,
            LITERAL_OWNERSHIP_FRAMEWORKS_STATUS,
        ],
    );
    assert_contains_all(
        "Frameworks 02 literal ownership status mirrors mirror",
        &read_repo(
            "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
        ),
        &[
            LITERAL_STATUS_MIRRORS_SLICE,
            LITERAL_STATUS_MIRRORS_STATUS,
            LITERAL_STATUS_MIRRORS_FRAMEWORKS_STATUS,
        ],
    );
}
