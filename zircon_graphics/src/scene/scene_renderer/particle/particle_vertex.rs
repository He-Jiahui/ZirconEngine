use bytemuck::{Pod, Zeroable};
use zircon_math::{RenderVec3, RenderVec4, Vec3, Vec4};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(in crate::scene::scene_renderer::particle) struct ParticleVertex {
    pub(in crate::scene::scene_renderer::particle) position: [f32; 3],
    pub(in crate::scene::scene_renderer::particle) color: [f32; 4],
}

impl ParticleVertex {
    pub(in crate::scene::scene_renderer::particle) fn new(position: Vec3, color: Vec4) -> Self {
        Self {
            position: RenderVec3::new(position.x, position.y, position.z).to_array(),
            color: RenderVec4::new(color.x, color.y, color.z, color.w).to_array(),
        }
    }

    pub(in crate::scene::scene_renderer::particle) fn layout() -> wgpu::VertexBufferLayout<'static>
    {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 12,
                    shader_location: 1,
                },
            ],
        }
    }
}
