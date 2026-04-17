use std::sync::Arc;

use crate::scene::resources::{GpuMeshResource, GpuTextureResource, PipelineKey};

pub(crate) struct MeshDraw {
    pub(crate) mesh: Arc<GpuMeshResource>,
    pub(crate) first_index: u32,
    pub(crate) draw_index_count: u32,
    pub(crate) indirect_args_buffer: Option<Arc<wgpu::Buffer>>,
    pub(crate) indirect_args_offset: u64,
    pub(crate) texture: Arc<GpuTextureResource>,
    pub(crate) pipeline_key: PipelineKey,
    #[allow(dead_code)]
    pub(crate) model_buffer: wgpu::Buffer,
    pub(crate) model_bind_group: wgpu::BindGroup,
}
