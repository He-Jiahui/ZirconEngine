use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct ModelUniform {
    pub(crate) model: [[f32; 4]; 4],
    pub(crate) tint: [f32; 4],
    pub(crate) shadow_params: [f32; 4],
    pub(crate) previous_model: [[f32; 4]; 4],
    pub(crate) motion_params: [f32; 4],
}
