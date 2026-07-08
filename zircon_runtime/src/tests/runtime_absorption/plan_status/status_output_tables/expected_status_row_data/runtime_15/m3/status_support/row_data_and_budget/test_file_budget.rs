type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 test file budget root-layout child split",
        &[
            "runtime_15_test_file_budget_root_layout_child_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/mod.rs",
            "structure_convention/test_file_budget/root_layout.rs",
            "runtime_15_test_file_budget_guard_is_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 test file budget root-layout status scan child split",
        &[
            "runtime_15_test_file_budget_root_layout_status_scan_child_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/root_layout.rs",
            "structure_convention/test_file_budget/root_layout/status_scan.rs",
            "runtime_15_test_file_budget_root_layout_status_scan_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 test file budget root-layout folder-backed guard child split",
        &[
            "runtime_15_test_file_budget_root_layout_folder_backed_guard_child_split_static_passed_cargo_timeout_no_result",
            "structure_convention/test_file_budget/root_layout.rs",
            "structure_convention/test_file_budget/root_layout/folder_backed.rs",
            "structure_convention/test_file_budget/root_layout/module_layout.rs",
            "runtime_15_test_file_budget_root_layout_folder_backed_guard_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 test file budget root-layout folder-backed support child-owner split",
        &[
            "runtime_15_test_file_budget_root_layout_folder_backed_support_child_owner_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/root_layout/folder_backed.rs",
            "structure_convention/test_file_budget/root_layout/folder_backed/assertions.rs",
            "structure_convention/test_file_budget/root_layout/folder_backed/guard_names.rs",
            "runtime_15_test_file_budget_root_layout_folder_backed_support_child_owners_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 test file budget root-layout assertions guard folder-backed split",
        &[
            "runtime_15_test_file_budget_root_layout_assertions_guard_folder_backed_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/root_layout/folder_backed/assertions.rs",
            "structure_convention/test_file_budget/root_layout/folder_backed/assertions_split.rs",
            "structure_convention/test_file_budget/root_layout/folder_backed/assertions/parent_mounts.rs",
            "structure_convention/test_file_budget/root_layout/folder_backed/assertions/render_status_children.rs",
            "runtime_15_test_file_budget_root_layout_assertions_guard_is_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 root-layout status-output Runtime 15 row-data child-source sync",
        &[
            "runtime_15_root_layout_status_output_runtime_15_row_data_child_source_sync_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/root_layout/folder_backed.rs",
            "structure_convention/test_file_budget/root_layout/folder_backed/sources.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/row_ownership.rs",
            "runtime_15_test_file_budget_guard_is_folder_backed",
            "runtime_15_status_output_runtime_15_row_data_is_child_owner",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 root entries/root-layout current-child route sync",
        &[
            "runtime_15_m3_root_entries_root_layout_current_child_route_sync_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/root_entries.rs",
            "structure_convention/test_file_budget/root_layout/module_layout.rs",
            "expected_slices/status/runtime_15/foundation/lock_poison.rs",
            "expected_slices/date/runtime_15/foundation/lock_poison.rs",
            "expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget/test_file_budget.rs",
            "runtime_15_root_entries_guard_child_owners_are_folder_backed",
            "runtime_15_test_file_budget_root_layout_folder_backed_guard_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 test file budget parent guard child-owner split",
        &[
            "runtime_15_test_file_budget_parent_guard_child_owner_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/mod.rs",
            "structure_convention/test_file_budget/core_framework.rs",
            "structure_convention/test_file_budget/ui_v2_asset.rs",
            "structure_convention/test_file_budget/ui_shared_core.rs",
            "structure_convention/test_file_budget/module_layout.rs",
            "runtime_15_test_file_budget_parent_guard_child_owner_split",
        ],
    ),
    (
        "Runtime 15 M3 historical oversized test roots closeout",
        &[
            "runtime_15_historical_oversized_test_roots_closeout_static_passed_cargo_deferred",
            "core/framework/tests.rs",
            "ui/tests/v2_asset.rs",
            "runtime_15_historical_oversized_test_roots_are_folder_backed",
        ],
    ),
];
