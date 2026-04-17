use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct GpuResidentProbeInput {
    pub(super) probe_id: u32,
    pub(super) slot: u32,
    pub(super) ray_budget: u32,
    pub(super) position_x_q: u32,
    pub(super) position_y_q: u32,
    pub(super) position_z_q: u32,
    pub(super) radius_q: u32,
    pub(super) previous_irradiance_rgb: u32,
    pub(super) parent_probe_id: u32,
    pub(super) _padding: u32,
}
