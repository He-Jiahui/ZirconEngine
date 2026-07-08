use super::*;

#[test]
fn runtime_15_expected_status_output_rows_accept_variable_evidence_anchors() {
    let expected_status_rows = read_runtime_src(EXPECTED_STATUS_ROWS_PATH);
    let runtime_15_m3_asset_budget = read_runtime_src(RUNTIME_15_M3_ASSET_BUDGET_ROW_DATA_PATH);
    let runtime_15_m3_status_support = read_runtime_src(RUNTIME_15_M3_STATUS_SUPPORT_ROW_DATA_PATH);
    let runtime_15_m4 = read_runtime_src(RUNTIME_15_M4_ROW_DATA_PATH);
    let runtime_15_m3_production_guard_support =
        read_runtime_src(RUNTIME_15_M3_PRODUCTION_GUARD_SUPPORT_ROW_DATA_PATH);

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
            "structure_convention/test_file_budget/runtime_diagnostics.rs",
            "structure_convention/test_file_budget/script_vm_tests.rs",
            "runtime_15_test_file_budget_guard_is_folder_backed",
        ],
    );
    assert_contains_all(
        "Runtime 15 M3 status support keeps multi-anchor evidence rows as slices",
        &runtime_15_m3_status_support,
        &[
            "Runtime 15 M3 test file budget root-layout status scan child split",
            "&[",
            "structure_convention/test_file_budget/root_layout/status_scan.rs",
            "runtime_15_test_file_budget_root_layout_status_scan_is_child_owner",
        ],
    );
    assert_contains_all(
        "Runtime 15 M4 keeps multi-anchor evidence rows as slices",
        &runtime_15_m4,
        &[
            "Runtime 15 M4 UI surface event-routing owner split",
            "&[",
            "ui/surface/surface/event_routing.rs",
            "ui/surface/surface/pointer_component_events.rs",
            "runtime_15_ui_surface_event_routing_is_child_owner",
        ],
    );
    assert_contains_all(
        "Runtime 15 M3 production support keeps variable evidence row",
        &runtime_15_m3_production_guard_support,
        &[
            VARIABLE_EVIDENCE_STATUS_NAME,
            VARIABLE_EVIDENCE_STATUS_ID,
            "plan_status/status_output_tables/expected_status_rows.rs",
            VARIABLE_EVIDENCE_GUARD_NAME,
        ],
    );
}
