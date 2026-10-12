#[path = "gpu_mesh_resource.rs"]
mod gpu_mesh_resource;
#[path = "resource_from_asset.rs"]
mod gpu_mesh_resource_from_asset;
#[path = "gpu_mesh_vertex.rs"]
mod gpu_mesh_vertex;
#[path = "vertex_from_mesh_vertex.rs"]
mod gpu_mesh_vertex_from_mesh_vertex;
#[path = "gpu_mesh_vertex_layout.rs"]
mod gpu_mesh_vertex_layout;
#[path = "mesh_bounds.rs"]
mod mesh_bounds;
#[path = "wire_segments.rs"]
mod wire_segments;

pub(crate) use gpu_mesh_resource::GpuMeshResource;
pub(crate) use gpu_mesh_vertex::GpuMeshVertex;
