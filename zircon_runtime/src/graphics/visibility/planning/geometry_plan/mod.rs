#[path = "build.rs"]
mod build;
#[path = "frontier/mod.rs"]
mod frontier;
#[path = "ordering/mod.rs"]
mod ordering;
#[path = "visibility/mod.rs"]
mod visibility;

pub(crate) use build::build_virtual_geometry_plan;

pub(crate) use ordering::cluster_ids_for_stable_instance_key;
