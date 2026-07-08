use super::*;

const STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/m3_m4_expected_slice_maps.rs";
const DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/m3_m4_expected_slice_maps.rs";

pub(in super::super) fn read_status_support_status_map_sources() -> String {
    read_status_support_map_sources(STATUS_MAP_PATH)
}

pub(in super::super) fn read_status_support_date_map_sources() -> String {
    read_status_support_map_sources(DATE_MAP_PATH)
}

fn read_status_support_map_sources(route_path: &str) -> String {
    let route_dir = route_path.trim_end_matches(".rs");
    let mut sources = vec![read_runtime_src(route_path)];
    sources.extend(
        [
            "expected_slice_guard_maps",
            "m3_row_data_maps",
            "m4_row_data_maps",
            "status_support_guard_maps",
        ]
        .iter()
        .map(|child| read_runtime_src(&format!("{route_dir}/{child}.rs"))),
    );
    sources.join("\n")
}
