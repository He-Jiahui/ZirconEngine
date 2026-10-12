#[path = "../../renderer/gpu/new/mod.rs"]
mod new;
#[path = "../../../cluster_cull_instance_work_items.rs"]
mod node_and_cluster_cull_instance_work_items;
#[path = "resources.rs"]
mod virtual_geometry_gpu_resources;

pub(crate) use virtual_geometry_gpu_resources::VirtualGeometryGpuResources;
