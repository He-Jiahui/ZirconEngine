use super::*;

#[test]
fn runtime_15_status_support_expected_slice_guard_body_status_mirrors_are_registered() {
    let status_rows = read_status_support_expected_slice_rows();
    let status_map = read_status_support_m3_m4_status_maps();
    let date_map = read_status_support_m3_m4_date_maps();

    assert_contains_all(
        "status-support guard body row data",
        &status_rows,
        &[
            GUARD_BODY_SLICE,
            GUARD_BODY_STATUS,
            GUARD_BODY_ROUTE_PATH,
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
    assert_contains_all(
        "status-support guard body status map",
        &status_map,
        &[GUARD_BODY_SLICE, GUARD_BODY_STATUS],
    );
    assert_contains_all(
        "status-support guard body date map",
        &date_map,
        &[GUARD_BODY_SLICE, "Some(\"2026-07-06\")"],
    );

    for (label, source) in [
        (
            "Runtime 15 plan",
            read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"),
        ),
        ("Runtime index", read_repo("docs/plans/zircon_runtime/runtime/index.md")),
        (
            "Frameworks 02",
            read_repo("docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md"),
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
            &[GUARD_BODY_SLICE, GUARD_BODY_STATUS, GUARD_BODY_GUARD],
        );
    }
    assert_contains_all(
        "Frameworks 02 status-support guard body mirror",
        &read_repo(
            "docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md",
        ),
        &[GUARD_BODY_FRAMEWORKS_STATUS],
    );
}
