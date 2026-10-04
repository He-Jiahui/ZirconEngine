use std::sync::Arc;

use crate::asset::{AssetReference, MeshAsset, ModelAsset, ModelPrimitiveAsset};
use crate::core::resource::ResourceId;

use super::super::GpuMeshResource;
use super::GpuModelResource;

impl GpuModelResource {
    pub(in crate::graphics::scene::resources) fn from_primitives(
        device: &wgpu::Device,
        id: ResourceId,
        primitives: Vec<ModelPrimitiveAsset>,
    ) -> Self {
        let mut meshes = Vec::with_capacity(primitives.len());
        for primitive in primitives {
            meshes.push(Arc::new(GpuMeshResource::from_asset(device, primitive)));
        }
        Self { id, meshes }
    }
}

/// 引用 mesh 只有加载及转换均成功时才替换内嵌 primitive；失败时保留模型提供的数据，供后续几何准备使用。
pub(in crate::graphics::scene::resources) fn model_primitives_preferring_mesh_assets<F>(
    asset: &ModelAsset,
    mut load_mesh_asset: F,
) -> Vec<ModelPrimitiveAsset>
where
    F: FnMut(&AssetReference) -> Option<MeshAsset>,
{
    let mut primitives = Vec::with_capacity(asset.primitives.len());
    for primitive in &asset.primitives {
        primitives.push(
            primitive
                .mesh
                .as_ref()
                .and_then(|reference| load_mesh_asset(reference))
                .and_then(|mesh| mesh.to_model_primitive().ok())
                .unwrap_or_else(|| primitive.clone()),
        );
    }
    primitives
}

#[cfg(test)]
#[path = "tests/gpu_model_resource_from_asset.rs"]
mod tests;
