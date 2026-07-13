use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_root_route_metadata_docs_are_mirrored() {
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let frameworks_02 = read_repo(
        "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
    );
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    let docs_required = [
        ROOT_ROUTE_METADATA_GUARD_SLICE,
        ROOT_ROUTE_METADATA_GUARD_STATUS,
        ROOT_ROUTE_METADATA_ROUTE_MOUNTS_SLICE,
        ROOT_ROUTE_METADATA_ROUTE_MOUNTS_STATUS,
        ROOT_ROUTE_METADATA_ROUTE_MOUNTS_FRAMEWORKS_STATUS,
        ROOT_ROUTE_METADATA_STATUS_MIRROR_GUARD_SLICE,
        ROOT_ROUTE_METADATA_STATUS_MIRROR_GUARD_STATUS,
        ROOT_ROUTE_METADATA_STATUS_MIRROR_GUARD_FRAMEWORKS_STATUS,
        ROOT_ROUTE_METADATA_ROUTE_PATH,
        ROOT_ROUTE_METADATA_ROUTE_MOUNTS_ROUTE_PATH,
        ROOT_ROUTE_METADATA_STATUS_MIRROR_GUARD_ROUTE_PATH,
        ROOT_ROUTE_METADATA_CHILDREN[0],
        ROOT_ROUTE_METADATA_CHILDREN[1],
        ROOT_ROUTE_METADATA_CHILDREN[2],
        ROOT_ROUTE_METADATA_ROUTE_MOUNTS_CHILDREN[0],
        ROOT_ROUTE_METADATA_ROUTE_MOUNTS_CHILDREN[1],
        ROOT_ROUTE_METADATA_STATUS_MIRROR_GUARD_CHILDREN[0],
        ROOT_ROUTE_METADATA_STATUS_MIRROR_GUARD_CHILDREN[1],
        ROOT_ROUTE_METADATA_STATUS_MIRROR_GUARD_CHILDREN[2],
        ROOT_ROUTE_METADATA_GUARD,
        ROOT_ROUTE_METADATA_ROUTE_MOUNTS_GUARD,
        ROOT_ROUTE_METADATA_STATUS_MIRROR_GUARD_GUARD,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("Frameworks 02", frameworks_02.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        assert_contains_all(label, source, &docs_required);
    }
    assert_contains_all(
        "Frameworks 02 review-guard root route status mirror",
        &frameworks_02,
        &[
            ROUTE_FRAMEWORKS_STATUS,
            ROOT_ROUTE_METADATA_GUARD_FRAMEWORKS_STATUS,
            ROOT_ROUTE_METADATA_ROUTE_MOUNTS_FRAMEWORKS_STATUS,
            ROOT_ROUTE_METADATA_STATUS_MIRROR_GUARD_FRAMEWORKS_STATUS,
        ],
    );
}
