use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_root_guard_body_route_mounts_status_is_mirrored() {
    let status_rows = read_review_guard_structure_rows();
    let status_map = read_status_review_foundation_sources();
    let date_map = read_date_review_foundation_sources();
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let frameworks_02 = read_repo(
        "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
    );
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "review guard route mounts row data",
        &status_rows,
        &[
            ROUTE_MOUNTS_SLICE,
            ROUTE_MOUNTS_STATUS,
            ROUTE_MOUNTS_ROUTE_PATH,
            ROUTE_MOUNTS_CHILDREN[0],
            ROUTE_MOUNTS_CHILDREN[1],
            ROUTE_MOUNTS_CHILDREN[2],
            ROUTE_MOUNTS_CHILDREN[3],
            ROUTE_MOUNTS_CHILDREN[4],
            ROUTE_MOUNTS_CHILDREN[5],
            ROUTE_MOUNTS_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "status review-guard route mounts map",
        &status_map,
        &[ROUTE_MOUNTS_SLICE, ROUTE_MOUNTS_STATUS],
    );
    assert_contains_all(
        "date review-guard route mounts map",
        &date_map,
        &[ROUTE_MOUNTS_SLICE, "Some(\"2026-07-06\")"],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("Frameworks 02", frameworks_02.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                ROUTE_MOUNTS_SLICE,
                ROUTE_MOUNTS_STATUS,
                ROUTE_MOUNTS_FRAMEWORKS_STATUS,
                ROUTE_MOUNTS_ROUTE_PATH,
                ROUTE_MOUNTS_CHILDREN[0],
                ROUTE_MOUNTS_CHILDREN[1],
                ROUTE_MOUNTS_CHILDREN[2],
                ROUTE_MOUNTS_CHILDREN[3],
                ROUTE_MOUNTS_CHILDREN[4],
                ROUTE_MOUNTS_CHILDREN[5],
                ROUTE_MOUNTS_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
}
