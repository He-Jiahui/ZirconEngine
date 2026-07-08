use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_typed_error_guard_body_status_is_synced() {
    let status_rows = read_status_support_expected_slice_rows();
    let status_map = read_status_review_typed_error_sources();
    let date_map = read_date_review_typed_error_sources();
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

    for (label, source) in [
        ("status-output expected-slice rows", status_rows.as_str()),
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
                "Runtime 15 M3 review guard typed-error expected-slice map child split",
                "runtime_15_review_guard_typed_error_expected_slice_map_child_split_static_passed_cargo_deferred",
                "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps.rs",
                "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/typed_error_maps.rs",
                "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps.rs",
                "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/typed_error_maps.rs",
                "runtime_15_review_guard_expected_slice_typed_error_maps_are_child_owned",
                "Cargo gate deferred",
            ],
        );
        assert_contains_all(
            label,
            source,
            &[
                GUARD_BODY_SLICE,
                GUARD_BODY_STATUS,
                GUARD_BODY_ROUTE_PATH,
                GUARD_BODY_CHILDREN[0],
                GUARD_BODY_CHILDREN[1],
                GUARD_BODY_CHILDREN[2],
                GUARD_BODY_CHILDREN[3],
                GUARD_BODY_CHILDREN[4],
                GUARD_BODY_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "typed-error expected-slice guard body status map",
        &status_map,
        &[GUARD_BODY_SLICE, GUARD_BODY_STATUS],
    );
    assert_contains_all(
        "typed-error expected-slice guard body date map",
        &date_map,
        &[GUARD_BODY_SLICE, "Some(\"2026-07-07\")"],
    );
    assert_contains_all(
        "Frameworks 02 typed-error guard-body mirror",
        &frameworks_02,
        &[GUARD_BODY_FRAMEWORKS_STATUS],
    );
}
