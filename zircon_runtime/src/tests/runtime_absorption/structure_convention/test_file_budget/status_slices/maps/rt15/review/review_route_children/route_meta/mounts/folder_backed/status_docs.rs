use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_route_metadata_route_mounts_folder_backed_status_is_mirrored(
) {
    let status_rows = read_status_support_expected_slice_rows();
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

    for (literal, mirrored) in [
        (
            "runtime_15_review_guard_expected_slice_route_metadata_route_mounts_folder_backed_guard_body_split_static_passed_cargo_deferred",
            REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_FOLDER_BACKED_STATUS,
        ),
        (
            "frameworks_02_m3_review_guard_expected_slice_route_metadata_route_mounts_folder_backed_guard_body_split_static_passed_cargo_deferred",
            REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_FOLDER_BACKED_FRAMEWORKS_STATUS,
        ),
    ] {
        assert_eq!(literal, mirrored);
    }

    for (label, source) in [
        ("status rows", status_rows.as_str()),
        ("status expected-slice review maps", status_map.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_FOLDER_BACKED_SLICE,
                REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_FOLDER_BACKED_STATUS,
            ],
        );
    }
    assert_contains_all(
        "date expected-slice review maps",
        &date_map,
        &[
            REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_FOLDER_BACKED_SLICE,
            "Some(\"2026-07-06\")",
        ],
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
                REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_FOLDER_BACKED_SLICE,
                REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_FOLDER_BACKED_STATUS,
                REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_FOLDER_BACKED_FRAMEWORKS_STATUS,
                REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_FOLDER_BACKED_ROUTE_PATH,
                REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_FOLDER_BACKED_CHILDREN[0],
                REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_FOLDER_BACKED_CHILDREN[1],
                REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_FOLDER_BACKED_CHILDREN[2],
                REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_FOLDER_BACKED_CHILDREN[3],
                REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_FOLDER_BACKED_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
}
