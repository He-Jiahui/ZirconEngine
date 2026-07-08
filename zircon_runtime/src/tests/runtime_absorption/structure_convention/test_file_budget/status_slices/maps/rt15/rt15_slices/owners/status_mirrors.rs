use super::*;

#[test]
fn runtime_15_expected_slice_child_owner_status_mirrors_stay_synced() {
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/base_maps.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    for (label, source) in [
        ("status-output M3 row data", status_rows.as_str()),
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
                "Runtime 15 M3 status output Runtime 15 expected-slice child-owner split",
                "runtime_15_status_output_runtime_15_expected_slice_child_owner_split_static_passed_cargo_deferred",
                "plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
                "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs",
                "runtime_15_status_output_runtime_15_expected_slice_maps_are_child_owners",
            ],
        );
    }
}
