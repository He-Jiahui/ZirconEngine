use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_root_guard_body_status_is_mirrored() {
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
                "Runtime 15 M3 review-guard expected-slice structure guard child-module split",
                "runtime_15_review_guard_expected_slice_structure_guard_child_module_split_static_passed_cargo_deferred",
                "Runtime 15 M3 structure-support expected-slice guard body child split",
                "runtime_15_structure_support_expected_slice_guard_body_child_split_static_passed_cargo_deferred",
                ROOT_GUARD_SLICE,
                ROOT_GUARD_STATUS,
                ROOT_GUARD_FRAMEWORKS_STATUS,
                ROOT_GUARD_ROUTE_PATH,
                ROOT_GUARD_CHILDREN[0],
                ROOT_GUARD_CHILDREN[1],
                ROOT_GUARD_CHILDREN[2],
                ROOT_GUARD_CHILDREN[3],
                ROOT_GUARD_CHILDREN[4],
                ROOT_GUARD_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
}
