use super::*;

#[path = "metadata/status_support_maps.rs"]
mod status_support_maps;

pub(in super::super) use status_support_maps::*;

pub(in super::super) const BUDGET_SLICE: &str =
    "Runtime 15 M3 status output Runtime 15 expected-slice child-owner budget route metadata child split";
pub(in super::super) const BUDGET_STATUS: &str =
    "runtime_15_status_output_runtime_15_expected_slice_child_owner_budget_route_metadata_child_split_static_passed_cargo_deferred";
pub(in super::super) const BUDGET_FRAMEWORKS_STATUS: &str =
    "frameworks_02_m3_status_output_runtime_15_expected_slice_child_owner_budget_route_metadata_child_split_static_passed_cargo_deferred";
pub(in super::super) const BUDGET_GUARD: &str =
    "runtime_15_status_output_runtime_15_expected_slice_child_owner_budget_route_metadata_is_child_owned";

pub(in super::super) const BUDGET_SOURCE_SLICE: &str =
    "Runtime 15 M3 status output Runtime 15 expected-slice child-owner budget source inventory folder-backed split";
pub(in super::super) const BUDGET_SOURCE_STATUS: &str =
    "runtime_15_status_output_runtime_15_expected_slice_child_owner_budget_source_inventory_folder_backed_static_passed_cargo_deferred";
pub(in super::super) const BUDGET_SOURCE_FRAMEWORKS_STATUS: &str =
    "frameworks_02_m3_status_output_runtime_15_expected_slice_child_owner_budget_source_inventory_folder_backed_static_passed_cargo_deferred";
pub(in super::super) const BUDGET_SOURCE_GUARD: &str =
    "runtime_15_status_output_runtime_15_expected_slice_child_owner_budget_sources_are_folder_backed";

pub(in super::super) const BUDGETS_ROUTE_PATH: &str =
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/owners/budgets.rs";
pub(in super::super) const BUDGETS_SOURCES_PATH: &str =
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/owners/budgets/sources.rs";
pub(in super::super) const BUDGETS_GUARD_BODY_PATH: &str =
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/owners/budgets/guard_body.rs";
pub(in super::super) const BUDGETS_ROUTE_METADATA_PATH: &str =
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/owners/budgets/route_metadata.rs";

pub(in super::super) const BUDGETS_SOURCES_CHILDREN: &[&str] = &[
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/owners/budgets/sources/budgets.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/owners/budgets/sources/doc_mirrors.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/owners/budgets/sources/folder_backed.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/owners/budgets/sources/metadata.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/owners/budgets/sources/source_paths.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/owners/budgets/sources/status_mirrors.rs",
];

pub(in super::super) const ROUTE_METADATA_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/route_metadata.rs";
pub(in super::super) const ROUTE_METADATA_ROW_CHILDREN: &[&str] = &[
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/route_meta/child_owner_budget_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/route_meta/child_owner_guard_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/route_meta/naming_boundary_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/route_meta/row_data_owner_rows.rs",
];
pub(in super::super) const STATUS_SUPPORT_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/m3_m4_expected_slice_maps.rs";
pub(in super::super) const STATUS_SUPPORT_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/m3_m4_expected_slice_maps.rs";

pub(in super::super) fn read_route_metadata_row_sources() -> String {
    std::iter::once(ROUTE_METADATA_ROWS_PATH)
        .chain(ROUTE_METADATA_ROW_CHILDREN.iter().copied())
        .map(read_runtime_src)
        .collect::<Vec<_>>()
        .join("\n")
}
