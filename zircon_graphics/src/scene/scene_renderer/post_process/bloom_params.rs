use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct BloomParams {
    pub(super) viewport: [u32; 4],
    pub(super) tuning: [f32; 4],
}
