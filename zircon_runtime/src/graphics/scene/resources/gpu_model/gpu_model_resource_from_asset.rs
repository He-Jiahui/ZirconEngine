use std::sync::Arc;

use crate::core::resource::ResourceId;

use super::super::GpuMeshResource;
use super::GpuModelResource;

impl GpuModelResource {
    pub(crate) fn from_asset(
        device: &wgpu::Device,
        id: ResourceId,
        asset: &crate::asset::assets::ModelAsset,
    ) -> Self {
        Self {
            id,
            meshes: asset
                .primitives
                .iter()
                .cloned()
                .map(|primitive| Arc::new(GpuMeshResource::from_asset(device, primitive)))
                .collect(),
        }
    }
}
