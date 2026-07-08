use super::*;

pub(super) fn read_structure_parent_route_source(path: &str) -> String {
    if path.ends_with("runtime07_script_maps.rs") {
        return read_runtime07_script_map_sources(path);
    }
    read_runtime_src(path)
}

fn read_runtime07_script_map_sources(route_path: &str) -> String {
    let route_dir = route_path.trim_end_matches(".rs");
    let mut sources = vec![read_runtime_src(route_path)];
    sources.extend(
        [
            "expected_slice_map_rows",
            "runtime07_guard_maps",
            "runtime07_owner_budget_maps",
            "runtime07_split_layout_maps",
            "script_vm_runtime_maps",
        ]
        .iter()
        .map(|child| read_runtime_src(&format!("{route_dir}/{child}.rs"))),
    );
    sources.join("\n")
}
