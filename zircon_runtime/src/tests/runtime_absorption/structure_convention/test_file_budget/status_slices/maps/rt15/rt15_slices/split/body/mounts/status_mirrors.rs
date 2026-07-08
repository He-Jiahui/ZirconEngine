use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_maps_guard_body_route_mounts_status_is_synced(
) {
    let row_data = read_top_level_support_row_sources();
    let status_map = read_status_support_status_map_sources();
    let date_map = read_status_support_date_map_sources();

    assert_contains_all(
        "Runtime 15 expected-slice maps guard-body route-mount row data",
        &row_data,
        &[
            ROUTE_MOUNTS_SLICE,
            ROUTE_MOUNTS_STATUS,
            "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/split/body/route_mounts.rs",
            GUARD_BODY_ROUTE_MOUNTS_CHILDREN[0],
            GUARD_BODY_ROUTE_MOUNTS_CHILDREN[6],
            ROUTE_MOUNTS_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "Runtime 15 expected-slice maps guard-body route-mount status/date maps",
        &format!("{status_map}\n{date_map}"),
        &[ROUTE_MOUNTS_SLICE, ROUTE_MOUNTS_STATUS, "2026-07-06"],
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
                ROUTE_MOUNTS_SLICE,
                ROUTE_MOUNTS_STATUS,
                ROUTE_MOUNTS_GUARD,
                "split/body/mounts/status_mirrors.rs",
            ],
        );
    }
    let frameworks = read_repo(
        "docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md",
    );
    assert_contains_all(
        "Frameworks 02 route-mount mirror",
        &frameworks,
        &[
            ROUTE_MOUNTS_SLICE,
            ROUTE_MOUNTS_STATUS,
            ROUTE_MOUNTS_FRAMEWORKS_STATUS,
        ],
    );
}
