use super::*;

#[test]
fn runtime_15_expected_status_output_rows_accept_variable_evidence_anchors() {
    let expected_status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_rows.rs",
    );
    let runtime_15_m3_asset_budget = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/asset_budget_tests.rs",
    );
    let runtime_15_m4 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs",
    );
    let runtime_15_m3_status_support = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs",
    );

    assert_contains_all(
        "expected status output slice type uses variable evidence anchors",
        &expected_status_rows,
        &[
            "pub(super) type ExpectedStatusOutputSlice",
            "(&'static str, &'static [&'static str])",
            "expected_status_output_slices()",
        ],
    );
    assert!(
        !expected_status_rows.contains("[&'static str; 4]"),
        "ExpectedStatusOutputSlice should not cap evidence anchors at four entries"
    );
    assert_contains_all(
        "Runtime 15 M3 keeps multi-anchor evidence rows as slices",
        &runtime_15_m3_asset_budget,
        &[
            "Runtime 15 M3 test file budget guard folder split",
            "&[",
            "test_file_budget/root_layout/status_scan.rs",
            "runtime_15_test_file_budget_guard_is_folder_backed",
        ],
    );
    assert_contains_all(
        "Runtime 15 M4 keeps multi-anchor evidence rows as slices",
        &runtime_15_m4,
        &[
            "Runtime 15 M4 UI surface event-routing owner split",
            "&[",
            "ui/surface/surface/event_routing/pointer_capture.rs",
            "runtime_15_ui_surface_event_routing_helpers_are_child_owners",
        ],
    );
}
