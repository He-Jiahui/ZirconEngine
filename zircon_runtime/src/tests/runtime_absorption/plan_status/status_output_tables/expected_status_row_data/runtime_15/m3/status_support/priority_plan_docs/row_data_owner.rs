type Slice = super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 status-support row-data owner child split",
        &[
            "runtime_15_status_support_row_data_owner_child_split_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/runtime_index_anchors.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs.rs",
            "runtime_15_status_support_row_data_owner_is_child_backed",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 priority plan docs row-data owner child split",
        &[
            "runtime_15_priority_plan_docs_row_data_owner_child_split_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/integrity_guards.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/owner_guards.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/status_followups.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/row_data_owner.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_status_support_priority_plan_docs.rs",
            "runtime_15_priority_plan_docs_row_data_owner_is_child_backed",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 priority plan docs owner-guard row-data child split",
        &[
            "runtime_15_priority_plan_docs_owner_guard_row_data_child_split_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/owner_guards.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/owner_guards/layout_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/owner_guards/inventory_rows.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_status_support_priority_plan_docs.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_status_support_priority_plan_docs/row_sources.rs",
            "runtime_15_priority_plan_docs_owner_guard_rows_are_child_owned",
            "Cargo gate deferred",
        ],
    ),
];
