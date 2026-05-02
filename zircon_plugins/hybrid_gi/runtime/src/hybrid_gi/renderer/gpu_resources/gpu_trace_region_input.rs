use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct GpuTraceRegionInput {
    pub(super) region_id: u32,
    pub(super) center_x_q: u32,
    pub(super) center_y_q: u32,
    pub(super) center_z_q: u32,
    pub(super) radius_q: u32,
    pub(super) coverage_q: u32,
    pub(super) rt_lighting_rgb: u32,
    pub(super) _padding1: u32,
}
