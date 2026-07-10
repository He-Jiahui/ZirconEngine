type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 status output variable evidence anchors",
        &[
            "runtime_15_status_output_variable_evidence_anchors_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_status_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs",
            "runtime_15_expected_status_output_rows_accept_variable_evidence_anchors",
        ],
    ),
    (
        "Runtime 15 M3 status output evidence anchors guard folder-backed split",
        &[
            "runtime_15_status_output_evidence_anchors_guard_folder_backed_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/row_data/evidence_anchors.rs",
            "structure_convention/test_file_budget/row_data/evidence_anchors/delegation.rs",
            "structure_convention/test_file_budget/row_data/evidence_anchors/variable_evidence.rs",
            "structure_convention/test_file_budget/row_data/evidence_anchors/status_mirrors.rs",
            "structure_convention/test_file_budget/row_data/evidence_anchors/budgets.rs",
            "runtime_15_status_output_evidence_anchors_guard_is_folder_backed",
            "runtime_15_expected_status_output_rows_accept_variable_evidence_anchors",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 evidence anchors status-mirror child split",
        &[
            "runtime_15_evidence_anchors_status_mirror_child_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/row_data/evidence_anchors.rs",
            "structure_convention/test_file_budget/row_data/evidence_anchors/status_mirrors.rs",
            "structure_convention/test_file_budget/row_data/evidence_anchors/status_mirrors/child_split_status.rs",
            "structure_convention/test_file_budget/row_data/evidence_anchors/status_mirrors/historical_status.rs",
            "structure_convention/test_file_budget/row_data/evidence_anchors/status_mirrors/folder_backed_status.rs",
            "runtime_15_evidence_anchors_status_mirror_children_are_child_owned",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 evidence anchors root inventory child split",
        &[
            "runtime_15_evidence_anchors_root_inventory_child_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/row_data/evidence_anchors.rs",
            "structure_convention/test_file_budget/row_data/evidence_anchors/root_paths.rs",
            "structure_convention/test_file_budget/row_data/evidence_anchors/root_statuses.rs",
            "structure_convention/test_file_budget/row_data/evidence_anchors/root_child_rows.rs",
            "structure_convention/test_file_budget/row_data/evidence_anchors/root_owner_paths.rs",
            "structure_convention/test_file_budget/row_data/evidence_anchors/root_inventory.rs",
            "runtime_15_evidence_anchors_root_inventory_is_child_owned",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 evidence anchors source/status-map sync",
        &[
            "runtime_15_evidence_anchors_source_status_map_sync_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/row_data/evidence_anchors/root_paths.rs",
            "structure_convention/test_file_budget/row_data/evidence_anchors/root_child_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/core_and_evidence/evidence_anchor_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget/test_file_budget.rs",
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/evidence_maps.rs",
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/root_layout_ui_maps.rs",
            "runtime_15_status_output_evidence_anchors_guard_is_folder_backed",
            "runtime_15_evidence_anchors_root_inventory_is_child_owned",
            "runtime_15_evidence_anchors_status_mirror_status_rows_are_current",
            "runtime_15_status_output_evidence_anchors_folder_backed_status_mirrors_are_current",
            "runtime_15_expected_status_output_rows_accept_variable_evidence_anchors",
            "Cargo gate deferred",
        ],
    ),
];
