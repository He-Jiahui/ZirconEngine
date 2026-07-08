use super::*;

pub(super) fn read_sources(paths: &[&str]) -> String {
    let mut sources = Vec::new();
    for path in paths {
        sources.push(read_runtime_src(path));
        if path.ends_with("status_support_maps/m3_m4_expected_slice_maps.rs") {
            let route_dir = path.trim_end_matches(".rs");
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
        }
    }
    sources.join("\n")
}
