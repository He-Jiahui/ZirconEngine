use super::Slice;

pub(super) const ROWS: [Slice; 3] = [
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
        "Runtime 15 M3 status output Runtime 15 expected-slice child-owner literal ownership folder-backed split",
        &[
            "runtime_15_status_output_runtime_15_expected_slice_child_owner_literal_ownership_folder_backed_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/child_owners/literal_ownership.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/child_owners/literal_ownership/budgets.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/child_owners/literal_ownership/date_literals.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/child_owners/literal_ownership/folder_backed.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/child_owners/literal_ownership/paths.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/child_owners/literal_ownership/source_groups.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/child_owners/literal_ownership/status_literals.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps/child_owners/literal_ownership/status_mirrors.rs",
            "runtime_15_status_output_runtime_15_expected_slice_child_owner_literal_ownership_is_folder_backed",
            "Cargo gate deferred",
        ],
    ),
];
