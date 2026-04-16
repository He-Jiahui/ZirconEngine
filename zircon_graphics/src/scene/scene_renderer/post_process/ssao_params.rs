use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct SsaoParams {
    pub(super) viewport_and_flags: [u32; 4],
    pub(super) tuning: [f32; 4],
}
