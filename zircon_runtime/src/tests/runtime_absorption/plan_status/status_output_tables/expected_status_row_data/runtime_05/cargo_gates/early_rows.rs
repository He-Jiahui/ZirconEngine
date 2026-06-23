use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 05 cargo-gates early Runtime 03 split",
        &[
            "cargo_gates/early/runtime_03.rs",
            "runtime_03_schedule_frame_loop_cargo_gate_stays_visible_until_schedule_validation",
            "early.rs",
            "plan-status support files 14/14",
        ],
    ),
    (
        "Runtime 05 cargo-gates early Runtime 01 split",
        &[
            "cargo_gates/early/runtime_01.rs",
            "runtime_01_tech_stack_cargo_gate_stays_visible_until_dependency_validation",
            "early.rs",
            "plan-status support files 14/14",
        ],
    ),
    (
        "Runtime 05 cargo-gates early Runtime 02 split",
        &[
            "cargo_gates/early/runtime_02.rs",
            "runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation",
            "early.rs",
            "plan-status support files 14/14",
        ],
    ),
    (
        "Runtime 05 cargo-gates early Runtime 04 split",
        &[
            "cargo_gates/early/runtime_04.rs",
            "runtime_04_asset_pipeline_cargo_gate_stays_visible_until_asset_validation",
            "early.rs",
            "plan-status support files 20/20",
        ],
    ),
    (
        "Runtime 05 cargo-gates early Runtime 06 split",
        &[
            "cargo_gates/early/runtime_06.rs",
            "runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation",
            "early.rs",
            "plan-status support files 20/20",
        ],
    ),
    (
        "Runtime 05 cargo-gates early Runtime 08 split",
        &[
            "cargo_gates/early/runtime_08.rs",
            "runtime_08_ecs_kernel_cargo_pending_gate_stays_explicit_until_ecs_validation",
            "early.rs",
            "plan-status support files 14/14",
        ],
    ),
    (
        "Runtime 05 cargo-gates early Runtime 07 split",
        &[
            "cargo_gates/early/runtime_07.rs",
            "runtime_07_performance_hotpath_cargo_gate_stays_visible_until_performance_validation",
            "early.rs",
            "plan-status support files 14/14",
        ],
    ),
    (
        "Runtime 05 status-output cargo-gates owner split",
        &[
            "expected_status_row_data/runtime_05/cargo_gates.rs",
            "runtime_05/cargo_gates/{early_rows,late_rows}.rs",
            "Runtime 05 cargo gate rows owner groups separately",
            "plan-status support files 76/76",
        ],
    ),
];
