use bytemuck::{Pod, Zeroable};

use crate::core::math::{RenderVec2, RenderVec3, RenderVec4, Vec2, Vec3, Vec4};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(in crate::graphics::scene::scene_renderer::sprite) struct SpriteVertex {
    position: [f32; 3],
    uv: [f32; 2],
    color: [f32; 4],
}

impl SpriteVertex {
    pub(in crate::graphics::scene::scene_renderer::sprite) fn new(
        position: Vec3,
        uv: Vec2,
        color: Vec4,
    ) -> Self {
        Self {
            position: RenderVec3::new(position.x, position.y, position.z).to_array(),
            uv: RenderVec2::new(uv.x, uv.y).to_array(),
            color: RenderVec4::new(color.x, color.y, color.z, color.w).to_array(),
        }
    }

    pub(in crate::graphics::scene::scene_renderer::sprite) fn layout(
    ) -> wgpu::VertexBufferLayout<'static> {
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
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 12,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 20,
                    shader_location: 2,
                },
            ],
        }
    }
}
