use super::*;

pub(super) const MAP_ROWS_SLICE: &str =
    "Runtime 15 M3 naming-boundary render-graphics expected-slice map rows folder-backed split";
pub(super) const MAP_ROWS_STATUS: &str = "runtime_15_naming_boundary_render_graphics_expected_slice_map_rows_folder_backed_static_passed_cargo_deferred";
pub(super) const MAP_ROWS_FRAMEWORKS_STATUS: &str = "frameworks_02_m3_naming_boundary_render_graphics_expected_slice_map_rows_folder_backed_static_passed_cargo_deferred";
pub(super) const MAP_ROWS_GUARD: &str =
    "runtime_15_status_output_naming_boundary_render_graphics_map_rows_are_folder_backed";

pub(super) const MAP_ROWS_GUARD_SLICE: &str =
    "Runtime 15 M3 naming-boundary render-graphics map rows guard folder-backed split";
pub(super) const MAP_ROWS_GUARD_STATUS: &str = "runtime_15_naming_boundary_render_graphics_map_rows_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const MAP_ROWS_GUARD_FRAMEWORKS_STATUS: &str = "frameworks_02_m3_naming_boundary_render_graphics_map_rows_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const MAP_ROWS_GUARD_GUARD: &str =
    "runtime_15_status_output_naming_boundary_render_graphics_map_rows_guard_is_folder_backed";

pub(super) const MAP_ROWS_GUARD_PARENT: &str = "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/naming_boundary/render_graphics_map_rows.rs";
pub(super) const NAMING_BOUNDARY_ROUTE_PARENT: &str = "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/naming_boundary.rs";

pub(super) const MAP_ROWS_GUARD_CHILDREN: &[&str] = &[
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/naming_boundary/render_graphics_map_rows/budgets.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/naming_boundary/render_graphics_map_rows/folder_backed.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/naming_boundary/render_graphics_map_rows/paths.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/naming_boundary/render_graphics_map_rows/route_mounts.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/naming_boundary/render_graphics_map_rows/status_mirrors.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/naming_boundary/render_graphics_map_rows/status_rows.rs",
];

pub(super) fn read_runtime_absorption_child(path: &str) -> String {
    read_runtime_src(&format!("tests/runtime_absorption/{path}"))
}

pub(super) fn read_guard_parent() -> String {
    read_runtime_absorption_child(MAP_ROWS_GUARD_PARENT)
}

pub(super) fn read_guard_children() -> String {
    MAP_ROWS_GUARD_CHILDREN
        .iter()
        .map(|path| read_runtime_absorption_child(path))
        .collect::<Vec<_>>()
        .join("\n")
}
