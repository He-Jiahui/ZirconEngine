use super::*;

pub(in super::super) fn read_status_support_status_map_sources() -> String {
    read_status_support_map_sources(STATUS_SUPPORT_STATUS_MAP_PATH)
}

pub(in super::super) fn read_status_support_date_map_sources() -> String {
    read_status_support_map_sources(STATUS_SUPPORT_DATE_MAP_PATH)
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
