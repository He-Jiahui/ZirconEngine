use std::sync::Arc;

use crate::asset::MeshAsset;

use super::super::GpuMeshResource;

pub(in crate::graphics::scene::resources) struct PreparedMesh {
    pub(in crate::graphics::scene::resources) revision: u64,
    pub(in crate::graphics::scene::resources) asset: Arc<MeshAsset>,
    pub(in crate::graphics::scene::resources) resource: Arc<GpuMeshResource>,
}
