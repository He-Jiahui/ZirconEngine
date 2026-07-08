use super::*;

pub(in super::super) const STATUS_STRUCTURE_ROUTE_MAP: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/structure_route_maps.rs";
pub(in super::super) const STATUS_STRUCTURE_ROUTE_MAP_CHILDREN: &[&str] = &[
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/structure_route_maps/structure_support_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/structure_route_maps/review_guard_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/structure_route_maps/naming_boundary_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/structure_route_maps/core_route_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/structure_route_maps/guard_rows.rs",
];
pub(in super::super) const DATE_STRUCTURE_ROUTE_MAP: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/structure_route_maps.rs";
pub(in super::super) const DATE_STRUCTURE_ROUTE_MAP_CHILDREN: &[&str] = &[
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/structure_route_maps/structure_support_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/structure_route_maps/review_guard_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/structure_route_maps/naming_boundary_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/structure_route_maps/core_route_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/structure_route_maps/guard_rows.rs",
];

pub(in super::super) fn read_status_structure_route_map_sources() -> String {
    read_structure_route_map_sources(
        STATUS_STRUCTURE_ROUTE_MAP,
        STATUS_STRUCTURE_ROUTE_MAP_CHILDREN,
    )
}

pub(in super::super) fn read_date_structure_route_map_sources() -> String {
    read_structure_route_map_sources(DATE_STRUCTURE_ROUTE_MAP, DATE_STRUCTURE_ROUTE_MAP_CHILDREN)
}

fn read_structure_route_map_sources(parent: &str, children: &[&str]) -> String {
    std::iter::once(parent)
        .chain(children.iter().copied())
        .map(read_runtime_src)
        .collect::<Vec<_>>()
        .join("\n")
}
