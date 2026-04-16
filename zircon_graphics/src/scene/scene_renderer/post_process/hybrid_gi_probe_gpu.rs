use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct GpuHybridGiProbe {
    pub(super) slot_and_budget: [f32; 4],
    pub(super) irradiance_and_weight: [f32; 4],
}
