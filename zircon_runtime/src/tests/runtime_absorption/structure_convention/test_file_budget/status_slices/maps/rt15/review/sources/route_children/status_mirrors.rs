use super::*;

#[test]
fn runtime_15_review_guard_root_source_route_children_status_mirrors_are_synced() {
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
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    for (label, source) in [
        ("status-output expected-slice rows", status_rows.as_str()),
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
                SOURCE_ROUTE_CHILDREN_SLICE,
                SOURCE_ROUTE_CHILDREN_STATUS,
                SOURCE_ROUTE_CHILDREN_FRAMEWORKS_STATUS,
                SOURCE_ROUTE_CHILDREN_ROUTE_PATH,
                SOURCE_ROUTE_CHILDREN_CHILDREN[0],
                SOURCE_ROUTE_CHILDREN_CHILDREN[1],
                SOURCE_ROUTE_CHILDREN_CHILDREN[2],
                SOURCE_ROUTE_CHILDREN_CHILDREN[3],
                SOURCE_ROUTE_CHILDREN_CHILDREN[4],
                SOURCE_ROUTE_CHILDREN_CHILDREN[5],
                SOURCE_ROUTE_CHILDREN_CHILDREN[6],
                SOURCE_ROUTE_CHILDREN_GUARD,
                "Cargo gate deferred",
            ],
        );
    }

    assert_contains_all(
        "review guard source route-children status map",
        &status_map,
        &[SOURCE_ROUTE_CHILDREN_SLICE, SOURCE_ROUTE_CHILDREN_STATUS],
    );
    assert_contains_all(
        "review guard source route-children date map",
        &date_map,
        &[SOURCE_ROUTE_CHILDREN_SLICE, "Some(\"2026-07-07\")"],
    );
}
