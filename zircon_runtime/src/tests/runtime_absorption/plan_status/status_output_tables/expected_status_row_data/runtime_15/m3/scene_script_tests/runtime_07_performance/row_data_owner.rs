type Slice = super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 scene-script Runtime 07 performance row-data child split",
        &[
            "runtime_15_scene_script_runtime_07_performance_row_data_child_split_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/runtime_07_performance.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/runtime_07_performance/primary_guard_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/runtime_07_performance/split_layout_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/runtime_07_performance/owner_budget_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests/runtime_07_performance/row_data_owner.rs",
            "runtime_15_scene_script_runtime_07_performance_row_data_is_child_backed",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 scene-script Runtime 07 performance guard folder-backed split",
        &[
            "runtime_15_scene_script_runtime_07_performance_guard_folder_backed_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/row_data/runtime_15_scene_script_row_data/runtime_07_performance.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_scene_script_row_data/runtime_07_performance/child_rows.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_scene_script_row_data/runtime_07_performance/export_chain.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_scene_script_row_data/runtime_07_performance/folder_backed.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_scene_script_row_data/runtime_07_performance/status_mirrors.rs",
            "runtime_15_scene_script_runtime_07_performance_guard_is_folder_backed",
            "Cargo gate deferred",
        ],
    ),
];
