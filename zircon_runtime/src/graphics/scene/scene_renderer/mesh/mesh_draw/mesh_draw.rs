use std::sync::Arc;

use crate::graphics::scene::resources::{GpuMeshResource, GpuTextureResource, PipelineKey};

use crate::graphics::types::VirtualGeometryPrepareClusterState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct VirtualGeometrySubmissionDetail {
    pub(crate) instance_index: Option<u32>,
    pub(crate) entity: u64,
    pub(crate) page_id: u32,
    pub(crate) submission_index: u32,
    pub(crate) draw_ref_rank: u32,
    pub(crate) draw_ref_index: u32,
    pub(crate) cluster_start_ordinal: u32,
    pub(crate) cluster_span_count: u32,
    pub(crate) cluster_total_count: u32,
    pub(crate) submission_slot: Option<u32>,
    pub(crate) state: VirtualGeometryPrepareClusterState,
    pub(crate) lineage_depth: u32,
    pub(crate) lod_level: u8,
    pub(crate) frontier_rank: u32,
}

pub(crate) struct MeshDraw {
    pub(crate) mesh: Arc<GpuMeshResource>,
    pub(crate) first_index: u32,
    pub(crate) draw_index_count: u32,
    pub(crate) indirect_args_buffer: Option<Arc<wgpu::Buffer>>,
    pub(crate) indirect_args_offset: u64,
    pub(crate) virtual_geometry_submission_key: Option<(u64, u32)>,
    pub(crate) virtual_geometry_submission_detail: Option<VirtualGeometrySubmissionDetail>,
    pub(crate) texture: Arc<GpuTextureResource>,
    pub(crate) pipeline_key: PipelineKey,
    #[allow(dead_code)]
    pub(crate) model_buffer: wgpu::Buffer,
    pub(crate) model_bind_group: wgpu::BindGroup,
}
