use std::sync::Arc;

use crate::graphics::scene::resources::{GpuMeshResource, GpuTextureResource, PipelineKey};

use super::virtual_geometry_submission_detail::VirtualGeometrySubmissionDetail;

pub(crate) struct MeshDraw {
    pub(super) mesh: Arc<GpuMeshResource>,
    pub(super) first_index: u32,
    pub(super) draw_index_count: u32,
    pub(super) indirect_args_buffer: Option<Arc<wgpu::Buffer>>,
    pub(super) indirect_args_offset: u64,
    pub(super) virtual_geometry_submission_key: Option<(u64, u32)>,
    pub(super) virtual_geometry_submission_detail: Option<VirtualGeometrySubmissionDetail>,
    pub(super) texture: Arc<GpuTextureResource>,
    pub(super) pipeline_key: PipelineKey,
    #[allow(dead_code)]
    pub(super) model_buffer: wgpu::Buffer,
    pub(super) model_bind_group: wgpu::BindGroup,
}

impl MeshDraw {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        mesh: Arc<GpuMeshResource>,
        first_index: u32,
        draw_index_count: u32,
        indirect_args_buffer: Option<Arc<wgpu::Buffer>>,
        indirect_args_offset: u64,
        virtual_geometry_submission_detail: Option<VirtualGeometrySubmissionDetail>,
        texture: Arc<GpuTextureResource>,
        pipeline_key: PipelineKey,
        model_buffer: wgpu::Buffer,
        model_bind_group: wgpu::BindGroup,
    ) -> Self {
        Self {
            mesh,
            first_index,
            draw_index_count,
            indirect_args_buffer,
            indirect_args_offset,
            virtual_geometry_submission_key: virtual_geometry_submission_detail
                .map(|detail| (detail.entity(), detail.page_id())),
            virtual_geometry_submission_detail,
            texture,
            pipeline_key,
            model_buffer,
            model_bind_group,
        }
    }
}
