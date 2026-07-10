use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_structure_guard_body_status_mirrors_are_registered() {
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

    assert_contains_all(
        "review guard structure body row data",
        &status_rows,
        &[
            ROOT_GUARD_SLICE,
            ROOT_GUARD_STATUS,
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
    assert_contains_all(
        "status review-guard structure body map",
        &status_map,
        &[ROOT_GUARD_SLICE, ROOT_GUARD_STATUS],
    );
    assert_contains_all(
        "date review-guard structure body map",
        &date_map,
        &[ROOT_GUARD_SLICE, "Some(\"2026-07-06\")"],
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
                ROOT_GUARD_SLICE,
                ROOT_GUARD_STATUS,
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
    assert_contains_all(
        "Frameworks 02 review-guard structure body status mirror",
        &frameworks_02,
        &[ROOT_GUARD_FRAMEWORKS_STATUS],
    );
}
