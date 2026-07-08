use super::*;
#[path = "sources/runtime07_script_maps.rs"]
mod runtime07_script_maps;
pub(super) fn read_sources(paths: &[&str]) -> String {
    paths
        .iter()
        .map(|path| runtime07_script_maps::read_structure_parent_route_source(path))
        .collect::<Vec<_>>()
        .join("\n")
}
pub(super) fn read_runtime_absorption_sources(paths: &[&str]) -> String {
    paths
        .iter()
        .map(|path| read_runtime_src(&format!("tests/runtime_absorption/{path}")))
        .collect::<Vec<_>>()
        .join("\n")
}
