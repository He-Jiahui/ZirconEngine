use super::*;

#[test]
fn runtime_15_review_guard_source_structure_paths_status_is_synced() {
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
        "review guard source structure paths row data",
        &status_rows,
        &[
            STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_SLICE,
            STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_STATUS,
            STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_ROUTE_PATH,
            STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_CHILDREN[0],
            STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_CHILDREN[1],
            STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_CHILDREN[2],
            STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_CHILDREN[3],
            STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_CHILDREN[4],
            STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_CHILDREN[5],
            STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_CHILDREN[6],
            STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_CHILDREN[7],
            STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_CHILDREN[8],
            STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "review guard source structure paths status/date maps",
        &format!("{status_map}\n{date_map}"),
        &[
            STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_SLICE,
            STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_STATUS,
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
                STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_SLICE,
                STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_STATUS,
                STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_ROUTE_PATH,
                STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_CHILDREN[0],
                STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_CHILDREN[1],
                STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_CHILDREN[2],
                STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_CHILDREN[3],
                STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_CHILDREN[4],
                STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_CHILDREN[5],
                STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_CHILDREN[6],
                STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_CHILDREN[7],
                STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_CHILDREN[8],
                STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "Frameworks 02 source structure paths mirror",
        &frameworks_02,
        &[STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_FRAMEWORKS_STATUS],
    );
}
