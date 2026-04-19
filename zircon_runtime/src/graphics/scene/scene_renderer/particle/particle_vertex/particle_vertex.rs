use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(in crate::graphics::scene::scene_renderer::particle) struct ParticleVertex {
    pub(in crate::graphics::scene::scene_renderer::particle) position: [f32; 3],
    pub(in crate::graphics::scene::scene_renderer::particle) color: [f32; 4],
}
