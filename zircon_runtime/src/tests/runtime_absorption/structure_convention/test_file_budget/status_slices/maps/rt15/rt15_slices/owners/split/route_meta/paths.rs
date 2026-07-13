use super::*;

pub(super) const ROUTE_METADATA_SLICE: &str = "Runtime 15 M3 status output Runtime 15 expected-slice child-owner guard route metadata folder-backed split";
pub(super) const ROUTE_METADATA_STATUS: &str = "runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_route_metadata_folder_backed_static_passed_cargo_deferred";
pub(super) const ROUTE_METADATA_FRAMEWORKS_STATUS: &str = "frameworks_02_m3_status_output_runtime_15_expected_slice_child_owner_guard_route_metadata_folder_backed_static_passed_cargo_deferred";
pub(super) const ROUTE_METADATA_GUARD: &str = "runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_route_metadata_is_folder_backed";

pub(super) const ROUTE_METADATA_CHILDREN: &[&str] = &[
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/owners/split/route_meta/budgets.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/owners/split/route_meta/doc_mirrors.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/owners/split/route_meta/folder_backed.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/owners/split/route_meta/paths.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/owners/split/route_meta/route_mounts.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/owners/split/route_meta/status_mirrors.rs",
];

pub(super) fn read_route_metadata_children() -> String {
    ROUTE_METADATA_CHILDREN
        .iter()
        .map(|path| read_runtime_src(&format!("tests/runtime_absorption/{path}")))
        .collect::<Vec<_>>()
        .join("\n")
}
