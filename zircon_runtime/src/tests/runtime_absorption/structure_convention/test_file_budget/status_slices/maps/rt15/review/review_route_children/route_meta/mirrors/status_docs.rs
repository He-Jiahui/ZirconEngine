use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_route_metadata_status_is_mirrored() {
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

    for (literal, mirrored) in [
        (
            "runtime_15_review_guard_expected_slice_route_metadata_status_mirrors_folder_backed_static_passed_cargo_deferred",
            REVIEW_ROUTE_METADATA_STATUS_MIRRORS_STATUS,
        ),
        (
            "frameworks_02_m3_review_guard_expected_slice_route_metadata_status_mirrors_folder_backed_static_passed_cargo_deferred",
            REVIEW_ROUTE_METADATA_STATUS_MIRRORS_FRAMEWORKS_STATUS,
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
        ("session note", session_note.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                REVIEW_ROUTE_METADATA_GUARD_SLICE,
                REVIEW_ROUTE_METADATA_GUARD_STATUS,
                REVIEW_ROUTE_METADATA_GUARD_FRAMEWORKS_STATUS,
                REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_SLICE,
                REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_STATUS,
                REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_FRAMEWORKS_STATUS,
                REVIEW_ROUTE_METADATA_STATUS_MIRRORS_SLICE,
                REVIEW_ROUTE_METADATA_STATUS_MIRRORS_STATUS,
                REVIEW_ROUTE_METADATA_STATUS_MIRRORS_FRAMEWORKS_STATUS,
                REVIEW_ROUTE_METADATA_ROUTE_PATH,
                REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_ROUTE_PATH,
                REVIEW_ROUTE_METADATA_STATUS_MIRRORS_ROUTE_PATH,
                REVIEW_ROUTE_METADATA_CHILDREN[0],
                REVIEW_ROUTE_METADATA_CHILDREN[1],
                REVIEW_ROUTE_METADATA_CHILDREN[2],
                REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_CHILDREN[0],
                REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_CHILDREN[1],
                REVIEW_ROUTE_METADATA_STATUS_MIRRORS_CHILDREN[0],
                REVIEW_ROUTE_METADATA_STATUS_MIRRORS_CHILDREN[1],
                REVIEW_ROUTE_METADATA_STATUS_MIRRORS_CHILDREN[2],
                REVIEW_ROUTE_METADATA_GUARD_GUARD,
                REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_GUARD,
                REVIEW_ROUTE_METADATA_STATUS_MIRRORS_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
}
