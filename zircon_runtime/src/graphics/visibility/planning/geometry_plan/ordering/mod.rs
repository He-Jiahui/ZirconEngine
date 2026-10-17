#[path = "../../../../planning/ordering/cluster_ids_for_entity.rs"]
mod cluster_ids_for_entity;
#[path = "../../../../planning/ordering/virtual_geometry_cluster_count.rs"]
mod virtual_geometry_cluster_count;
#[path = "../../../../ordering/virtual_geometry_cluster_ordinal.rs"]
mod virtual_geometry_cluster_ordinal;
#[path = "../../../../ordering/virtual_geometry_cluster_sort_key.rs"]
mod virtual_geometry_cluster_sort_key;

pub(crate) use cluster_ids_for_entity::cluster_ids_for_stable_instance_key;
pub(in crate::graphics::visibility::planning::build_virtual_geometry_plan) use virtual_geometry_cluster_count::virtual_geometry_cluster_count;
pub(in crate::graphics::visibility::planning::build_virtual_geometry_plan) use virtual_geometry_cluster_ordinal::virtual_geometry_cluster_ordinal;
pub(in crate::graphics::visibility::planning::build_virtual_geometry_plan) use virtual_geometry_cluster_sort_key::virtual_geometry_cluster_sort_key;
