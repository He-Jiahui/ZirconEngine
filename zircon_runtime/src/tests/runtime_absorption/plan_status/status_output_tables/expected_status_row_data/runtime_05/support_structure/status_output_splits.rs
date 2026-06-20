use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 05 status-output row-data group split",
        [
            "EXPECTED_STATUS_OUTPUT_SLICE_GROUPS",
            "expected_status_output_slices",
            "expected_status_row_data/runtime_05.rs",
            "plan-status support files 32/32",
        ],
    ),
    (
        "Runtime 05 status-output Runtime 05 row-data family split",
        [
            "expected_status_row_data/runtime_05.rs",
            "runtime_05/scene_closeout.rs",
            "runtime_05/audit_metadata.rs",
            "plan-status support files 52/52",
        ],
    ),
    (
        "Runtime 05 status-output Runtime 14 row-data family split",
        [
            "expected_status_row_data/runtime_14.rs",
            "runtime_14/audit_sync.rs",
            "runtime_14/cargo_gates.rs",
            "plan-status support files 55/55",
        ],
    ),
    (
        "Runtime 05 status-output Runtime 07 row-data family split",
        [
            "runtime_06_09/runtime_07.rs",
            "runtime_07/performance.rs",
            "runtime_07/owner_budget.rs",
            "plan-status support files 59/59",
        ],
    ),
    (
        "Runtime 05 status-output Runtime 09 row-data family split",
        [
            "runtime_06_09/runtime_09.rs",
            "runtime_06_09/runtime_09/{baseline,layout_pipeline,legacy_renames}.rs",
            "Runtime 09 owner groups separately",
            "plan-status support files 62/62",
        ],
    ),
    (
        "Runtime 05 status-output status/date owner split",
        [
            "status_output_tables/expected_slices.rs",
            "expected_slices/status.rs",
            "expected_slices/date.rs",
            "plan-status support files 46/46",
        ],
    ),
    (
        "Runtime 05 status-output Runtime 01-04 row-data group split",
        [
            "expected_status_row_data/runtime_01_04.rs",
            "runtime_01_04/runtime_01.rs",
            "runtime_01_04/runtime_04.rs",
            "plan-status support files 44/44",
        ],
    ),
    (
        "Runtime 05 status-output Runtime 06-09 row-data group split",
        [
            "expected_status_row_data/runtime_06_09.rs",
            "runtime_06_09/runtime_06.rs",
            "runtime_06_09/runtime_09.rs",
            "plan-status support files 36/36",
        ],
    ),
    (
        "Runtime 05 status-output Runtime 10-13 row-data group split",
        [
            "expected_status_row_data/runtime_10_13.rs",
            "runtime_10_13/runtime_10.rs",
            "runtime_10_13/runtime_13.rs",
            "plan-status support files 40/40",
        ],
    ),
    (
        "Runtime 05 status-output Runtime 10 row-data family split",
        [
            "runtime_10_13/runtime_10.rs",
            "runtime_10/{dynamic_api,session,ui_contract}.rs",
            "Runtime 10 owner groups separately",
            "plan-status support files 65/65",
        ],
    ),
    (
        "Runtime 05 status-output Runtime 12 row-data family split",
        [
            "runtime_10_13/runtime_12.rs",
            "runtime_12/{baseline,action_mapping,gamepad,host_recording}.rs",
            "Runtime 12 owner groups separately",
            "plan-status support files 69/69",
        ],
    ),
    (
        "Runtime 05 status-output support-structure owner split",
        [
            "expected_status_row_data/runtime_05/support_structure.rs",
            "runtime_05/support_structure/{plan_status_modules,status_output_splits}.rs",
            "Runtime 05 support rows owner groups separately",
            "plan-status support files 71/71",
        ],
    ),
    (
        "Runtime 05 status-output scene-closeout owner split",
        [
            "expected_status_row_data/runtime_05/scene_closeout.rs",
            "runtime_05/scene_closeout/{dynamic_scene_rows,full_scene_gate_rows,source_guard_rows}.rs",
            "Runtime 05 scene closeout rows owner groups separately",
            "plan-status support files 79/79",
        ],
    ),
];
