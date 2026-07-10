use super::*;

#[test]
fn runtime_15_review_guard_foundation_status_date_maps_docs_are_mirrored() {
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

    let docs_required = [
        REVIEW_FOUNDATION_MAPS_SLICE,
        REVIEW_FOUNDATION_MAPS_STATUS,
        REVIEW_FOUNDATION_MAPS_GUARD,
        REVIEW_FOUNDATION_MAP_GUARD_SLICE,
        REVIEW_FOUNDATION_MAP_GUARD_STATUS,
        REVIEW_FOUNDATION_MAP_GUARD_ROUTE_PATH,
        REVIEW_FOUNDATION_MAP_GUARD_CHILDREN[0],
        REVIEW_FOUNDATION_MAP_GUARD_CHILDREN[1],
        REVIEW_FOUNDATION_MAP_GUARD_GUARD,
        REVIEW_FOUNDATION_ROUTE_MOUNT_GUARD_SLICE,
        REVIEW_FOUNDATION_ROUTE_MOUNT_GUARD_STATUS,
        REVIEW_FOUNDATION_ROUTE_MOUNT_GUARD_ROUTE_PATH,
        REVIEW_FOUNDATION_ROUTE_MOUNT_GUARD_CHILDREN[0],
        REVIEW_FOUNDATION_ROUTE_MOUNT_GUARD_CHILDREN[1],
        REVIEW_FOUNDATION_ROUTE_MOUNT_GUARD_GUARD,
        REVIEW_FOUNDATION_STATUS_MIRROR_GUARD_SLICE,
        REVIEW_FOUNDATION_STATUS_MIRROR_GUARD_STATUS,
        REVIEW_FOUNDATION_STATUS_MIRROR_GUARD_ROUTE_PATH,
        REVIEW_FOUNDATION_STATUS_MIRROR_GUARD_CHILDREN[0],
        REVIEW_FOUNDATION_STATUS_MIRROR_GUARD_CHILDREN[1],
        REVIEW_FOUNDATION_STATUS_MIRROR_GUARD_CHILDREN[2],
        REVIEW_FOUNDATION_STATUS_MIRROR_GUARD_GUARD,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("Frameworks 02", frameworks_02.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
    ] {
        assert_contains_all(label, source, &docs_required);
    }
    assert_contains_all(
        "Frameworks 02 review foundation map mirror",
        &frameworks_02,
        &[
            REVIEW_FOUNDATION_MAPS_FRAMEWORKS_STATUS,
            REVIEW_FOUNDATION_MAP_GUARD_FRAMEWORKS_STATUS,
            REVIEW_FOUNDATION_ROUTE_MOUNT_GUARD_FRAMEWORKS_STATUS,
            REVIEW_FOUNDATION_STATUS_MIRROR_GUARD_FRAMEWORKS_STATUS,
        ],
    );
}
