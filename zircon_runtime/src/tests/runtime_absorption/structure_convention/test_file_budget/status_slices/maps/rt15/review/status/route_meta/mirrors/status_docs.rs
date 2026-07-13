use super::*;

#[test]
fn runtime_15_status_support_expected_slice_route_metadata_docs_are_registered() {
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let frameworks_02 = read_repo(
        "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
    );
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    for (literal, mirrored) in [
        (
            "runtime_15_status_support_expected_slice_route_metadata_status_mirrors_folder_backed_static_passed_cargo_deferred",
            ROUTE_METADATA_STATUS_MIRRORS_STATUS,
        ),
        (
            "frameworks_02_m3_status_support_expected_slice_route_metadata_status_mirrors_folder_backed_static_passed_cargo_deferred",
            ROUTE_METADATA_STATUS_MIRRORS_FRAMEWORKS_STATUS,
        ),
    ] {
        assert_eq!(literal, mirrored);
    }
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
                ROUTE_METADATA_SLICE,
                ROUTE_METADATA_STATUS,
                ROUTE_METADATA_STATUS_MIRRORS_SLICE,
                ROUTE_METADATA_STATUS_MIRRORS_STATUS,
                ROUTE_METADATA_ROUTE_PATH,
                ROUTE_METADATA_STATUS_MIRRORS_ROUTE_PATH,
                ROUTE_METADATA_CHILDREN[4],
                ROUTE_METADATA_STATUS_MIRRORS_CHILDREN[0],
                ROUTE_METADATA_STATUS_MIRRORS_CHILDREN[1],
                ROUTE_METADATA_STATUS_MIRRORS_CHILDREN[2],
                ROUTE_METADATA_GUARD,
                ROUTE_METADATA_STATUS_MIRRORS_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "Frameworks 02 status-support route metadata status-mirror mirror",
        &frameworks_02,
        &[
            ROUTE_METADATA_FRAMEWORKS_STATUS,
            ROUTE_METADATA_STATUS_MIRRORS_FRAMEWORKS_STATUS,
        ],
    );
}
