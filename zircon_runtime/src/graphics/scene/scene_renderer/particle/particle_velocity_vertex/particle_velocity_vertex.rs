use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(in crate::graphics::scene::scene_renderer::particle) struct ParticleVelocityVertex {
    pub(in crate::graphics::scene::scene_renderer::particle) current_position: [f32; 3],
    pub(in crate::graphics::scene::scene_renderer::particle) previous_position: [f32; 3],
}
