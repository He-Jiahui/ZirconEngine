use std::sync::Arc;

use crate::scene::resources::{GpuMeshResource, GpuTextureResource, PipelineKey};

pub(crate) struct MeshDraw {
    pub(crate) mesh: Arc<GpuMeshResource>,
    pub(crate) texture: Arc<GpuTextureResource>,
    pub(crate) pipeline_key: PipelineKey,
    #[allow(dead_code)]
    pub(crate) model_buffer: wgpu::Buffer,
    pub(crate) model_bind_group: wgpu::BindGroup,
}

impl MeshDraw {
    pub(crate) fn is_transparent(&self) -> bool {
        self.pipeline_key.alpha_blend
    }
}
