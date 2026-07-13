use super::*;

pub(super) const FOUNDATION_MAP_SLICE: &str =
    "Runtime 15 M3 foundation expected-slice maps folder-backed split";
pub(super) const FOUNDATION_MAP_STATUS: &str =
    "runtime_15_foundation_expected_slice_maps_folder_backed_static_passed_cargo_deferred";
pub(super) const FOUNDATION_MAP_FRAMEWORKS_STATUS: &str =
    "frameworks_02_m3_foundation_expected_slice_maps_folder_backed_static_passed_cargo_deferred";
pub(super) const FOUNDATION_MAP_GUARD: &str =
    "runtime_15_foundation_expected_slice_maps_are_folder_backed";

pub(super) const FOUNDATION_GUARD_SLICE: &str =
    "Runtime 15 M3 foundation expected-slice maps guard folder-backed split";
pub(super) const FOUNDATION_GUARD_STATUS: &str =
    "runtime_15_foundation_expected_slice_maps_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const FOUNDATION_GUARD_FRAMEWORKS_STATUS: &str = "frameworks_02_m3_foundation_expected_slice_maps_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const FOUNDATION_GUARD: &str =
    "runtime_15_foundation_expected_slice_maps_guard_is_folder_backed";

pub(super) const STATUS_PARENT: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/foundation.rs";
pub(super) const DATE_PARENT: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/foundation.rs";
pub(super) const FOUNDATION_ROUTE_PARENT: &str = "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/foundation.rs";
pub(super) const EXPECTED_SLICE_MAPS_PARENT: &str = "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/runtime_15_expected_slice_maps.rs";

pub(super) const PLAN_STATUS_CHILDREN: &[&str] = &[
    "foundation/asset_provider_cleanup.rs",
    "foundation/core_cleanup.rs",
    "foundation/graphics_diagnostics.rs",
    "foundation/lock_poison.rs",
    "foundation/map_rows.rs",
    "foundation/typed_error_core.rs",
    "foundation/typed_error_plugin.rs",
];

pub(super) const FOUNDATION_GUARD_CHILDREN: &[&str] = &[
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/foundation/budgets.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/foundation/child_sources.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/foundation/folder_backed.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/foundation/paths.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/foundation/route_mounts.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/foundation/status_mirrors.rs",
];
pub(super) const FOUNDATION_STATUS_MIRRORS_PARENT: &str = "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/foundation/status_mirrors.rs";
pub(super) const FOUNDATION_STATUS_MIRRORS_CHILDREN: &[&str] = &[
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/foundation/mirrors/budgets.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/foundation/mirrors/docs.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/foundation/mirrors/folder_backed.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/foundation/mirrors/paths.rs",
    "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/foundation/mirrors/row_data.rs",
];

pub(super) fn read_runtime_absorption_child(path: &str) -> String {
    read_runtime_src(&format!("tests/runtime_absorption/{path}"))
}

pub(super) fn read_plan_status_child_sources(root: &str) -> String {
    PLAN_STATUS_CHILDREN
        .iter()
        .map(|child| read_runtime_src(&format!("{root}/{child}")))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn read_top_level_support_row_sources() -> String {
    [
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/top_level_support.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/top_level_support/maps_guard_rows.rs",
    ]
    .iter()
    .map(|path| read_runtime_src(path))
    .collect::<Vec<_>>()
    .join("\n")
}

pub(super) fn read_foundation_status_mirror_parent() -> String {
    read_runtime_absorption_child(FOUNDATION_STATUS_MIRRORS_PARENT)
}

pub(super) fn read_foundation_status_mirror_children() -> String {
    FOUNDATION_STATUS_MIRRORS_CHILDREN
        .iter()
        .map(|path| read_runtime_absorption_child(path))
        .collect::<Vec<_>>()
        .join("\n")
}
