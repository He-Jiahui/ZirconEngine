use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 05 cargo-gates late Runtime 10 split",
        &[
            "cargo_gates/late/runtime_10.rs",
            "runtime_10_m1_3_cargo_pending_gate_stays_explicit_until_dynamic_api_validation",
            "runtime_10_ui_contract_m2_gate_stays_pending_until_runtime_09_owner_handoff",
            "plan-status support files 14/14",
        ],
    ),
    (
        "Runtime 05 cargo-gates late Runtime 11 split",
        &[
            "cargo_gates/late/runtime_11.rs",
            "runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass",
            "late.rs",
            "plan-status support files 15/15",
        ],
    ),
    (
        "Runtime 05 cargo-gates late Runtime 12 split",
        &[
            "cargo_gates/late/runtime_12.rs",
            "runtime_12_input_stack_cargo_pending_gate_stays_explicit_until_input_validation",
            "late.rs",
            "plan-status support files 16/16",
        ],
    ),
    (
        "Runtime 05 cargo-gates late Runtime 13 split",
        &[
            "cargo_gates/late/runtime_13.rs",
            "runtime_13_script_binding_cargo_gate_stays_visible_until_script_filters_pass",
            "late.rs",
            "plan-status support files 18/18",
        ],
    ),
    (
        "Runtime 05 cargo-gates late Runtime 14 split",
        &[
            "cargo_gates/late/runtime_14.rs",
            "runtime_14_module_family_cargo_gate_stays_visible_until_filters_pass",
            "late.rs",
            "plan-status support files 18/18",
        ],
    ),
];
