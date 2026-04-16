use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct ClusteredDirectionalLight {
    pub(super) direction: [f32; 4],
    pub(super) color_intensity: [f32; 4],
}
