type Slice = super::ExpectedStatusOutputSlice;

#[path = "base_maps/core_rows.rs"]
mod core_rows;

const REMAINING_ROWS: [Slice; 4] = [
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

const COMBINED_ROWS: [Slice; 7] = [
    core_rows::ROWS[0],
    core_rows::ROWS[1],
    core_rows::ROWS[2],
    REMAINING_ROWS[0],
    REMAINING_ROWS[1],
    REMAINING_ROWS[2],
    REMAINING_ROWS[3],
];

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &COMBINED_ROWS;
