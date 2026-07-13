use super::*;

#[path = "sources/status_support_maps.rs"]
mod status_support_maps;

pub(super) use status_support_maps::*;

pub(super) const SLICE: &str =
    "Runtime 15 M3 status output Runtime 15 expected-slice maps guard folder-backed split";
pub(super) const STATUS: &str = "runtime_15_status_output_runtime_15_expected_slice_maps_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const FRAMEWORKS_STATUS: &str = "frameworks_02_m3_status_output_runtime_15_expected_slice_maps_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const GUARD: &str =
    "runtime_15_status_output_runtime_15_expected_slice_maps_guard_is_folder_backed";

pub(super) const PARENT_PATH: &str = "structure_convention/test_file_budget/status_slices/maps/rt15/runtime_15_expected_slice_maps.rs";
pub(super) const CHILD_OWNER_PATH: &str =
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/child_owners.rs";
pub(super) const CHILD_OWNER_DOC_ANCHOR: &str = "rt15_slices/child_owners.rs";
pub(super) const NAMING_BOUNDARY_PATH: &str =
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/naming_boundary.rs";
pub(super) const NAMING_BOUNDARY_SOURCES_PATH: &str = "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/naming_boundary/sources.rs";
pub(super) const NAMING_BOUNDARY_GUARD_BODY_PATH: &str = "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/naming_boundary/guard_body.rs";
pub(super) const NAMING_BOUNDARY_ROUTE_METADATA_PATH: &str = "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/naming_boundary/route_metadata.rs";
pub(super) const SPLIT_LAYOUT_PATH: &str =
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/split_layout.rs";
pub(super) const SPLIT_LAYOUT_SOURCES_PATH: &str =
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/split/sources.rs";
pub(super) const SPLIT_LAYOUT_GUARD_BODY_PATH: &str =
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/split/guard_body.rs";
pub(super) const SPLIT_LAYOUT_STATUS_MIRRORS_PATH: &str = "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/split/status_mirrors.rs";

const STRUCTURE_READ_ROOT: &str = "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices";
const TOP_LEVEL_SUPPORT_ROWS_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/top_level_support.rs";
const TOP_LEVEL_SUPPORT_ROW_CHILDREN: &[&str] = &[
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/top_level_support/support_layout_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/top_level_support/child_owner_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/top_level_support/maps_guard_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/top_level_support/naming_boundary_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/top_level_support/row_data_owner_rows.rs",
];

pub(super) fn read_runtime_15_map(relative_path: &str) -> String {
    read_runtime_src(&format!("{STRUCTURE_READ_ROOT}/{relative_path}"))
}

pub(super) fn read_runtime_15_map_parent() -> String {
    read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/runtime_15_expected_slice_maps.rs",
    )
}

pub(super) fn read_top_level_support_row_sources() -> String {
    std::iter::once(TOP_LEVEL_SUPPORT_ROWS_PATH)
        .chain(TOP_LEVEL_SUPPORT_ROW_CHILDREN.iter().copied())
        .map(read_runtime_src)
        .collect::<Vec<_>>()
        .join("\n")
}
