type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 production guard core-and-evidence row-data folder-backed split",
        &[
            "runtime_15_production_guard_core_and_evidence_row_data_folder_backed_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/core_and_evidence.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/core_and_evidence/production_file_budget_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/core_and_evidence/evidence_anchor_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/core_and_evidence/child_group_row_data_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/core_and_evidence/child_group_inventory_rows.rs",
            "Cargo gate deferred active Render Plan08/text lanes",
        ],
    ),
    (
        "Runtime 15 M3 production file budget guard child-owner split",
        &[
            "runtime_15_production_file_budget_guard_child_owner_split_static_passed_cargo_deferred",
            "structure_convention/production_file_budget.rs",
            "structure_convention/production_file_budget/module_layout.rs",
            "runtime_15_production_file_budget_guard_child_owner_split",
        ],
    ),
    (
        "Runtime 15 M3 render shader template assembly guard WGSL contracts split",
        &[
            "runtime_15_render_shader_template_assembly_guard_wgsl_contracts_split_static_passed_cargo_deferred",
            "structure_convention/production_file_budget/render_shader_template_assembly.rs",
            "structure_convention/production_file_budget/render_shader_template_assembly/wgsl_contracts.rs",
            "runtime_15_render_shader_template_wgsl_contracts_are_child_owner",
        ],
    ),
];
