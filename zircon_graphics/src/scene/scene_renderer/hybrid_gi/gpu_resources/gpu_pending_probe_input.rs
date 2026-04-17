use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct GpuPendingProbeInput {
    pub(super) probe_id: u32,
    pub(super) logical_index: u32,
    pub(super) ray_budget: u32,
    pub(super) position_x_q: u32,
    pub(super) position_y_q: u32,
    pub(super) position_z_q: u32,
    pub(super) radius_q: u32,
    pub(super) parent_probe_id: u32,
    pub(super) resident_ancestor_probe_id: u32,
    pub(super) resident_ancestor_depth: u32,
}
