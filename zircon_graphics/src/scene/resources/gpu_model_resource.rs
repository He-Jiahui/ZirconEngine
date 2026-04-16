use std::sync::Arc;

use zircon_resource::ResourceId;

use super::gpu_mesh_resource::GpuMeshResource;

pub(crate) struct GpuModelResource {
    #[allow(dead_code)]
    pub(crate) id: ResourceId,
    pub(crate) meshes: Vec<Arc<GpuMeshResource>>,
}
