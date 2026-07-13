use super::*;

#[test]
fn runtime_15_structure_support_expected_slice_parent_route_paths_status_mirrors_are_synced() {
    let status_rows = read_structure_support_expected_slice_rows();
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    for (label, source) in [
        ("status-output expected-slice rows", status_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                PARENT_ROUTE_PATHS_SLICE,
                PARENT_ROUTE_PATHS_STATUS,
                PARENT_ROUTE_PATHS_FRAMEWORKS_STATUS,
                "structure_convention/test_file_budget/status_slices/maps/rt15/review/structure/parent_routes/paths.rs",
                "structure_convention/test_file_budget/status_slices/maps/rt15/review/structure/parent_routes/paths/status_metadata.rs",
                "structure_convention/test_file_budget/status_slices/maps/rt15/review/structure/parent_routes/paths/route_inputs.rs",
                "structure_convention/test_file_budget/status_slices/maps/rt15/review/structure/parent_routes/paths/child_guard_paths.rs",
                PARENT_ROUTE_PATHS_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
}
