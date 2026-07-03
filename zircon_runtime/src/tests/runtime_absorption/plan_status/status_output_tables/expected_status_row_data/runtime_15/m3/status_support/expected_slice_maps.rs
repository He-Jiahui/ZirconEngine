type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 status output expected-slice maps split",
        &[
            "runtime_15_status_output_expected_slice_maps_split_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_slices/status.rs",
            "plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
            "runtime_15_status_output_expected_slice_maps_are_child_owners",
        ],
    ),
    (
        "Runtime 15 M3 status output Runtime 15 expected-slice child-owner split",
        &[
            "runtime_15_status_output_runtime_15_expected_slice_child_owner_split_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs",
            "runtime_15_status_output_runtime_15_expected_slice_maps_are_child_owners",
        ],
    ),
    (
        "Runtime 15 M3 status output expected-slice guard maps child-owner split",
        &[
            "runtime_15_status_output_expected_slice_guard_maps_child_owner_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/status_output_expected_slices/maps.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics.rs",
            "runtime_15_status_output_expected_slice_guard_maps_are_child_owners",
        ],
    ),
    (
        "Runtime 15 M3 status output expected-slice top-level map support child-owner split",
        &[
            "runtime_15_status_output_expected_slice_top_level_map_support_child_owner_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/assertions.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/sources.rs",
            "runtime_15_status_output_expected_slice_top_level_map_support_child_owners_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 structure-support expected-slice map child-owner split",
        &[
            "runtime_15_structure_support_expected_slice_map_child_owner_split_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs",
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps.rs",
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/naming_guard_maps.rs",
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs",
            "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs",
            "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps.rs",
            "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/naming_guard_maps.rs",
            "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs",
            "runtime_15_structure_support_expected_slice_maps_are_child_owners",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 structure-convention warning cleanup",
        &[
            "runtime_15_structure_convention_warning_cleanup_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/historical_oversized_roots.rs",
            "structure_convention/test_file_budget/status_output_row_data/evidence_anchors.rs",
            "structure_convention/test_file_budget/code_review_findings/status_docs/source_anchors.rs",
            "structure_convention/test_file_budget/root_layout/folder_backed/assertions.rs",
            "structure_convention/test_file_budget/rhi_command_list.rs",
            "runtime_15_rhi_command_list_tests_are_folder_backed",
            "runtime_15_rhi_device_contract_tests_are_folder_backed",
            "runtime_15_runtime_diagnostics_tests_are_folder_backed",
        ],
    ),
];
