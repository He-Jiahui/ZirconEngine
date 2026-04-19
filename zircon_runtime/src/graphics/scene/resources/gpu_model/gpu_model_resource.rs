use std::sync::Arc;

use crate::core::resource::ResourceId;

use super::super::GpuMeshResource;

pub(crate) struct GpuModelResource {
    #[allow(dead_code)]
    pub(crate) id: ResourceId,
    pub(crate) meshes: Vec<Arc<GpuMeshResource>>,
}
