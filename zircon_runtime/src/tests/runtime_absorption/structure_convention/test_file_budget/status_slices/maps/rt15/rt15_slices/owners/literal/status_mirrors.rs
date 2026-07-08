use super::*;

#[test]
fn runtime_15_expected_slice_child_literal_ownership_status_mirrors_are_synced() {
    let status_rows = read_runtime_src(BASE_EXPECTED_SLICE_ROWS);
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
        "Runtime 15 expected-slice literal row data",
        &status_rows,
        &[
            LITERAL_OWNERSHIP_SLICE,
            LITERAL_OWNERSHIP_STATUS,
            LITERAL_OWNERSHIP_ROUTE_PATH,
            LITERAL_OWNERSHIP_CHILDREN[0],
            LITERAL_OWNERSHIP_CHILDREN[1],
            LITERAL_OWNERSHIP_CHILDREN[2],
            LITERAL_OWNERSHIP_CHILDREN[3],
            LITERAL_OWNERSHIP_CHILDREN[4],
            LITERAL_OWNERSHIP_CHILDREN[5],
            LITERAL_OWNERSHIP_CHILDREN[6],
            LITERAL_OWNERSHIP_GUARD,
            "Cargo gate deferred",
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
                LITERAL_OWNERSHIP_SLICE,
                LITERAL_OWNERSHIP_STATUS,
                LITERAL_OWNERSHIP_ROUTE_PATH,
                LITERAL_OWNERSHIP_CHILDREN[0],
                LITERAL_OWNERSHIP_CHILDREN[1],
                LITERAL_OWNERSHIP_CHILDREN[2],
                LITERAL_OWNERSHIP_CHILDREN[3],
                LITERAL_OWNERSHIP_CHILDREN[4],
                LITERAL_OWNERSHIP_CHILDREN[5],
                LITERAL_OWNERSHIP_CHILDREN[6],
                LITERAL_OWNERSHIP_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "Frameworks 02 literal ownership mirror",
        &frameworks_02,
        &[LITERAL_OWNERSHIP_FRAMEWORKS_STATUS],
    );
}
