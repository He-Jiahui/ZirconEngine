use super::*;

#[test]
fn runtime_15_structure_support_expected_slice_parent_route_guard_body_route_metadata_docs_are_synced(
) {
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let frameworks_plan = read_repo(
        "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
    );
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("Frameworks 02", frameworks_plan.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                PARENT_ROUTE_GUARD_BODY_ROUTE_METADATA_SLICE,
                PARENT_ROUTE_GUARD_BODY_ROUTE_METADATA_STATUS,
                PARENT_ROUTE_GUARD_BODY_ROUTE_METADATA_ROUTE_PATH,
                PARENT_ROUTE_GUARD_BODY_ROUTE_METADATA_CHILDREN[0],
                PARENT_ROUTE_GUARD_BODY_ROUTE_METADATA_CHILDREN[1],
                PARENT_ROUTE_GUARD_BODY_ROUTE_METADATA_CHILDREN[2],
                PARENT_ROUTE_GUARD_BODY_ROUTE_METADATA_CHILDREN[3],
                PARENT_ROUTE_GUARD_BODY_ROUTE_METADATA_CHILDREN[4],
                PARENT_ROUTE_GUARD_BODY_ROUTE_METADATA_CHILDREN[5],
                PARENT_ROUTE_GUARD_BODY_ROUTE_METADATA_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "Frameworks 02 parent-route guard-body route metadata status mirror",
        &frameworks_plan,
        &[
            PARENT_ROUTE_GUARD_BODY_FRAMEWORKS_STATUS,
            PARENT_ROUTE_GUARD_BODY_ROUTE_METADATA_FRAMEWORKS_STATUS,
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                PARENT_ROUTE_GUARD_BODY_SLICE,
                PARENT_ROUTE_GUARD_BODY_STATUS,
                PARENT_ROUTE_GUARD_BODY_ROUTE_PATH,
                PARENT_ROUTE_GUARD_BODY_CHILDREN[0],
                PARENT_ROUTE_GUARD_BODY_CHILDREN[1],
                PARENT_ROUTE_GUARD_BODY_CHILDREN[2],
                PARENT_ROUTE_GUARD_BODY_CHILDREN[3],
                PARENT_ROUTE_GUARD_BODY_CHILDREN[4],
                PARENT_ROUTE_GUARD_BODY_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
}
