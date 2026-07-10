type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 child-groups root inventory child split",
        &[
            "runtime_15_m3_child_groups_root_inventory_child_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/row_data/runtime_15_m3_child_groups.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_m3_child_groups/root_paths.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_m3_child_groups/root_statuses.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_m3_child_groups/root_child_rows.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_m3_child_groups/root_owner_paths.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_m3_child_groups/root_inventory.rs",
            "runtime_15_m3_child_groups_root_inventory_is_child_owned",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 child-groups exports child split",
        &[
            "runtime_15_m3_child_groups_exports_child_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/row_data/runtime_15_m3_child_groups/exports.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_m3_child_groups/exports/top_level.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_m3_child_groups/exports/runtime_15_parent.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_m3_child_groups/exports/runtime_15_m3_parent.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_m3_child_groups/exports/status_mirrors.rs",
            "runtime_15_m3_child_groups_exports_status_mirrors_are_current",
            "Cargo gate deferred",
        ],
    ),
];
