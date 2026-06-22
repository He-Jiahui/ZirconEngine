use std::sync::Arc;

use crate::core::resource::ResourceId;

use super::super::GpuMeshResource;

pub(crate) struct GpuModelResource {
    pub(super) id: ResourceId,
    pub(crate) meshes: Vec<Arc<GpuMeshResource>>,
}

impl GpuModelResource {
    pub(crate) const fn id(&self) -> ResourceId {
        self.id
    }
}
