use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct SceneUniform {
    pub(crate) view_proj: [[f32; 4]; 4],
    pub(crate) light_dir: [f32; 4],
    pub(crate) light_color: [f32; 4],
    pub(crate) ambient_color: [f32; 4],
}
