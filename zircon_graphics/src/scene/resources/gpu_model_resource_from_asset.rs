use std::sync::Arc;

use zircon_resource::ResourceId;

use super::gpu_mesh_resource::GpuMeshResource;
use super::gpu_model_resource::GpuModelResource;

impl GpuModelResource {
    pub(super) fn from_asset(
        device: &wgpu::Device,
        id: ResourceId,
        asset: zircon_asset::ModelAsset,
    ) -> Self {
        Self {
            id,
            meshes: asset
                .primitives
                .into_iter()
                .map(|primitive| Arc::new(GpuMeshResource::from_asset(device, primitive)))
                .collect(),
        }
    }
}
