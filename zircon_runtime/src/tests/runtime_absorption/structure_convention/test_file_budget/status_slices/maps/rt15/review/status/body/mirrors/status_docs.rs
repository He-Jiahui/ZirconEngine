use super::*;

#[test]
fn runtime_15_status_support_expected_slice_guard_body_status_mirrors_status_is_synced() {
    let status_rows = read_status_support_expected_slice_rows();
    let status_map = read_status_support_m3_m4_status_maps();
    let date_map = read_status_support_m3_m4_date_maps();

    assert_contains_all(
        "status-support guard body status-mirror row data",
        &status_rows,
        &[
            GUARD_BODY_STATUS_MIRRORS_SLICE,
            GUARD_BODY_STATUS_MIRRORS_STATUS,
            GUARD_BODY_STATUS_MIRRORS_ROUTE_PATH,
            GUARD_BODY_STATUS_MIRROR_CHILDREN[0],
            GUARD_BODY_STATUS_MIRROR_CHILDREN[1],
            GUARD_BODY_STATUS_MIRROR_CHILDREN[2],
            GUARD_BODY_STATUS_MIRROR_CHILDREN[3],
            GUARD_BODY_STATUS_MIRROR_CHILDREN[4],
            GUARD_BODY_STATUS_MIRRORS_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "status-support guard body status-mirror status/date maps",
        &format!("{status_map}\n{date_map}"),
        &[
            GUARD_BODY_STATUS_MIRRORS_SLICE,
            GUARD_BODY_STATUS_MIRRORS_STATUS,
            "2026-07-06",
        ],
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
                GUARD_BODY_STATUS_MIRRORS_SLICE,
                GUARD_BODY_STATUS_MIRRORS_STATUS,
                "status/body/status_mirrors.rs",
                "status/body/mirrors/status_docs.rs",
                GUARD_BODY_STATUS_MIRRORS_GUARD,
            ],
        );
    }
    assert_contains_all(
        "Frameworks 02 status-support guard body status-mirror mirror",
        &read_repo(
            "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
        ),
        &[
            GUARD_BODY_STATUS_MIRRORS_SLICE,
            GUARD_BODY_STATUS_MIRRORS_STATUS,
            GUARD_BODY_STATUS_MIRRORS_FRAMEWORKS_STATUS,
        ],
    );
}
