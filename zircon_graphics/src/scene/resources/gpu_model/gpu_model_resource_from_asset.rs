use std::sync::Arc;

use zircon_resource::ResourceId;

use super::super::GpuMeshResource;
use super::GpuModelResource;

impl GpuModelResource {
    pub(crate) fn from_asset(
        device: &wgpu::Device,
        id: ResourceId,
        asset: zircon_asset::assets::ModelAsset,
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
