use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct ModelUniform {
    pub(crate) model: [[f32; 4]; 4],
    pub(crate) tint: [f32; 4],
}
