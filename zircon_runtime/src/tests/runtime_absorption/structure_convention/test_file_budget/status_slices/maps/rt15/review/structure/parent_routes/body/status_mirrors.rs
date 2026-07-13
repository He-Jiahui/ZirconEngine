use super::*;

#[test]
fn runtime_15_structure_support_expected_slice_parent_route_status_mirrors_are_synced() {
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
                "Runtime 15 M3 structure-support expected-slice parent maps folder-backed split",
                "runtime_15_structure_support_expected_slice_parent_maps_folder_backed_static_passed_cargo_deferred",
                "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/structure_route_maps.rs",
                "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/row_data_owner_maps.rs",
                "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/runtime07_script_maps.rs",
                "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/plugin_export_gameplay_maps.rs",
                "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/structure_route_maps.rs",
                "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/row_data_owner_maps.rs",
                "structure_convention/test_file_budget/status_slices/maps/rt15/review/structure/parent_route_children.rs",
                "runtime_15_structure_support_expected_slice_parent_maps_are_folder_backed",
                "Cargo gate deferred",
            ],
        );
    }
}
