use super::*;

#[test]
fn runtime_15_review_guard_source_status_maps_status_mirrors_are_synced() {
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

    for (label, source) in [
        ("status-output expected-slice rows", status_rows.as_str()),
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
                SOURCE_STATUS_MAPS_SLICE,
                SOURCE_STATUS_MAPS_STATUS,
                SOURCE_STATUS_MAPS_FRAMEWORKS_STATUS,
                SOURCE_STATUS_MAPS_ROUTE_PATH,
                SOURCE_STATUS_MAPS_CHILDREN[0],
                SOURCE_STATUS_MAPS_CHILDREN[1],
                SOURCE_STATUS_MAPS_CHILDREN[2],
                SOURCE_STATUS_MAPS_CHILDREN[3],
                SOURCE_STATUS_MAPS_CHILDREN[4],
                SOURCE_STATUS_MAPS_CHILDREN[5],
                SOURCE_STATUS_MAPS_CHILDREN[6],
                SOURCE_STATUS_MAPS_CHILDREN[7],
                SOURCE_STATUS_MAPS_CHILDREN[8],
                SOURCE_STATUS_MAPS_CHILDREN[9],
                SOURCE_STATUS_MAPS_CHILDREN[10],
                SOURCE_STATUS_MAPS_CHILDREN[11],
                SOURCE_STATUS_MAPS_GUARD,
                "Cargo gate deferred",
            ],
        );
    }

    assert_contains_all(
        "review guard source status-map status map",
        &status_map,
        &[SOURCE_STATUS_MAPS_SLICE, SOURCE_STATUS_MAPS_STATUS],
    );
    assert_contains_all(
        "review guard source status-map date map",
        &date_map,
        &[SOURCE_STATUS_MAPS_SLICE, "Some(\"2026-07-07\")"],
    );
    assert_contains_all(
        "Frameworks 02 source status-map mirror",
        &frameworks_02,
        &[SOURCE_STATUS_MAPS_FRAMEWORKS_STATUS],
    );
}
