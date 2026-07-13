use super::*;

#[test]
fn runtime_15_status_support_review_guard_row_data_route_guard_is_folder_backed() {
    let status_rows = read_status_support_expected_slice_rows();
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
                "Runtime 15 M3 status-support review-guard row-data route expected-slice guard folder-backed split",
                "runtime_15_status_support_review_guard_row_data_route_expected_slice_guard_folder_backed_static_passed_cargo_deferred",
                "Runtime 15 M3 status-support review-guard row-data route guard route-input folder-backed split",
                "runtime_15_status_support_review_guard_row_data_route_guard_route_input_folder_backed_static_passed_cargo_deferred",
                "Runtime 15 M3 status-support review-guard row-data expected-slice maps folder-backed split",
                "runtime_15_status_support_review_guard_row_data_expected_slice_maps_folder_backed_static_passed_cargo_deferred",
                "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/row_data_maps/review_guard_row_data_maps/base_child_owner_maps.rs",
                "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/row_data_maps/review_guard_row_data_maps/direct_assertion_maps.rs",
                "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/row_data_maps/review_guard_row_data_maps/base_child_owner_maps.rs",
                "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/row_data_maps/review_guard_row_data_maps/direct_assertion_maps.rs",
                "structure_convention/test_file_budget/status_slices/maps/rt15/review/status/review_guard_row_data_route_children.rs",
                "structure_convention/test_file_budget/status_slices/maps/rt15/review/status/review_row_routes/status_mirrors.rs",
                "structure_convention/test_file_budget/status_slices/maps/rt15/review/status/review_row_routes/route_inputs.rs",
                "structure_convention/test_file_budget/status_slices/maps/rt15/review/status/review_row_routes/child_paths.rs",
                "structure_convention/test_file_budget/status_slices/maps/rt15/review/status/review_row_routes/source_reads.rs",
                "runtime_15_status_support_review_guard_row_data_route_guard_is_folder_backed",
                "runtime_15_status_support_review_guard_row_data_route_guard_route_inputs_are_child_owned",
                "runtime_15_status_support_review_guard_row_data_expected_slice_maps_are_folder_backed",
                "Cargo gate deferred",
            ],
        );
    }
}
