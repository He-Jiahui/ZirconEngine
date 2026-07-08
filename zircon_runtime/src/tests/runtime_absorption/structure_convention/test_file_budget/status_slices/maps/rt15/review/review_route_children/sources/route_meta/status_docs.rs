use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_route_metadata_source_constants_status_is_synced() {
    let status_rows = read_status_support_expected_slice_rows();
    let status_map = read_status_review_foundation_sources();
    let date_map = read_date_review_foundation_sources();
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let frameworks_02 = read_repo(
        "docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md",
    );
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    assert_contains_all(
        "review-route route metadata source constants row data",
        &status_rows,
        &[
            REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_SLICE,
            REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_STATUS,
            REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_ROUTE_PATH,
            REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_CHILDREN[0],
            REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_CHILDREN[1],
            REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_CHILDREN[2],
            REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_CHILDREN[3],
            REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_CHILDREN[4],
            REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_CHILDREN[5],
            REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_CHILDREN[6],
            REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_CHILDREN[7],
            REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "review-route route metadata source constants status/date maps",
        &format!("{status_map}\n{date_map}"),
        &[
            REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_SLICE,
            REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_STATUS,
            "2026-07-06",
        ],
    );
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("Frameworks 02", frameworks_02.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_SLICE,
                REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_STATUS,
                REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_ROUTE_PATH,
                REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_CHILDREN[0],
                REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_CHILDREN[1],
                REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_CHILDREN[2],
                REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_CHILDREN[3],
                REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_CHILDREN[4],
                REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_CHILDREN[5],
                REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_CHILDREN[6],
                REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_CHILDREN[7],
                REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "Frameworks 02 review-route route metadata source constants mirror",
        &frameworks_02,
        &[REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_FRAMEWORKS_STATUS],
    );
}
