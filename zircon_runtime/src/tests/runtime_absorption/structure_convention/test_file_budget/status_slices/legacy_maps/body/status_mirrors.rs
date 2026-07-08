use super::*;

#[test]
fn runtime_15_status_output_expected_slice_legacy_guard_body_status_is_synced() {
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/expected_slice_guards.rs",
    );
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/expected_slice_support_maps.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/expected_slice_support_maps.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let framework_plan = read_repo(
        "docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md",
    );
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    assert_contains_all(
        "legacy guard-body row data",
        &status_rows,
        &[
            LEGACY_GUARD_BODY_SLICE,
            LEGACY_GUARD_BODY_STATUS,
            LEGACY_GUARD_BODY_PARENT,
            LEGACY_GUARD_BODY_CHILDREN[0],
            LEGACY_GUARD_BODY_CHILDREN[1],
            LEGACY_GUARD_BODY_CHILDREN[2],
            LEGACY_GUARD_BODY_CHILDREN[3],
            LEGACY_GUARD_BODY_CHILDREN[4],
            LEGACY_GUARD_BODY_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "legacy guard-body status map",
        &status_map,
        &[LEGACY_GUARD_BODY_SLICE, LEGACY_GUARD_BODY_STATUS],
    );
    assert_contains_all(
        "legacy guard-body date map",
        &date_map,
        &[LEGACY_GUARD_BODY_SLICE, "Some(\"2026-07-07\")"],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("framework plan", framework_plan.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                LEGACY_GUARD_BODY_SLICE,
                LEGACY_GUARD_BODY_STATUS,
                LEGACY_GUARD_BODY_FRAMEWORKS_STATUS,
                LEGACY_GUARD_BODY_PARENT,
                LEGACY_GUARD_BODY_CHILDREN[0],
                LEGACY_GUARD_BODY_CHILDREN[1],
                LEGACY_GUARD_BODY_CHILDREN[2],
                LEGACY_GUARD_BODY_CHILDREN[3],
                LEGACY_GUARD_BODY_CHILDREN[4],
                LEGACY_GUARD_BODY_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
}
