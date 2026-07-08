type Slice = super::Slice;

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
    (
        "Runtime 15 M3 runtime-15 expected-slice topic guard child-module split",
        &[
            "runtime_15_runtime_15_expected_slice_topic_guard_child_module_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/status_output_expected_slices/maps.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps.rs",
            "runtime_15_status_output_runtime_15_expected_slice_maps_are_child_owners",
            "runtime_15_status_output_expected_slice_guard_maps_are_child_owners",
            "Cargo gate deferred active Render Plan08 lane",
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
        "Runtime 15 M3 status-output expected-slice guard maps folder-backed split",
        &[
            "runtime_15_status_output_expected_slice_guard_maps_folder_backed_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/status_output_expected_slices/maps.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/guard_body.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/guard_body/budgets.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/guard_body/child_ownership.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/guard_body/folder_backed.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/guard_body/paths.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/guard_body/route_mounts.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/guard_body/status_mirrors.rs",
            "runtime_15_status_output_expected_slice_guard_maps_is_folder_backed",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 expected-slice module-layout guard body child split",
        &[
            "runtime_15_expected_slice_module_layout_guard_body_child_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/status_output_expected_slices/module_layout.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/module_layout/guard_body.rs",
            "runtime_15_status_output_expected_slice_guard_child_owner_split",
            "Cargo gate deferred active Render Plan08 lane",
        ],
    ),
];
