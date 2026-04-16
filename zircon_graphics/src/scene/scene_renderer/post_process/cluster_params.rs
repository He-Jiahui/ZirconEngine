use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct ClusterParams {
    pub(super) viewport_and_clusters: [u32; 4],
    pub(super) counts: [u32; 4],
    pub(super) strengths: [f32; 4],
}
