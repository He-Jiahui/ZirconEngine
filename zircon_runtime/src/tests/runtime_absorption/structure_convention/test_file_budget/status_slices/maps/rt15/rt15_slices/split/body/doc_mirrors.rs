use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_maps_guard_body_docs_are_synced() {
    super::super::status_mirrors::assert_status_rows_and_docs_are_synced();

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
                GUARD_BODY_SLICE,
                GUARD_BODY_STATUS,
                SPLIT_LAYOUT_GUARD_BODY_PATH,
                GUARD_BODY_CHILDREN[0],
                GUARD_BODY_CHILDREN[1],
                GUARD_BODY_CHILDREN[2],
                GUARD_BODY_CHILDREN[3],
                GUARD_BODY_CHILDREN[4],
                GUARD_BODY_CHILDREN[5],
                GUARD_BODY_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "Frameworks 02 expected-slice maps guard-body mirror",
        &frameworks_02,
        &[FRAMEWORKS_STATUS, GUARD_BODY_FRAMEWORKS_STATUS],
    );
}
