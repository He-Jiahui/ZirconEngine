use super::*;

#[path = "structure_paths/folder_backed.rs"]
mod folder_backed;
#[path = "structure_paths/foundation_maps.rs"]
mod foundation_maps;
#[path = "structure_paths/review_route.rs"]
mod review_route;
#[path = "structure_paths/root_routes.rs"]
mod root_routes;
#[path = "structure_paths/root_sources.rs"]
mod root_sources;
#[path = "structure_paths/route_metadata.rs"]
mod route_metadata;
#[path = "structure_paths/status_docs.rs"]
mod status_docs;
#[path = "structure_paths/structure_support.rs"]
mod structure_support;
#[path = "structure_paths/typed_status_support.rs"]
mod typed_status_support;

pub(in super::super) use foundation_maps::*;
pub(in super::super) use review_route::*;
pub(in super::super) use root_routes::*;
pub(in super::super) use root_sources::*;
pub(in super::super) use route_metadata::*;
pub(in super::super) use structure_support::*;
pub(in super::super) use typed_status_support::*;

pub(in super::super) fn read_review_structure_path_sources(paths: &[&str]) -> String {
    paths
        .iter()
        .map(|path| read_runtime_src(&format!("tests/runtime_absorption/{path}")))
        .collect::<Vec<_>>()
        .join("\n")
}
