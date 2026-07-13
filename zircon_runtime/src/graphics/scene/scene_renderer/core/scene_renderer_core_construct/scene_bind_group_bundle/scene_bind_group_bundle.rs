use super::super::super::scene_renderer_core::{SceneEnvironmentBrdfLut, SceneEnvironmentCubemap};

pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_construct) struct SceneBindGroupBundle
{
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_construct) layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_construct) uniform_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_construct) environment_sh9_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_construct) environment_cubemap:
        SceneEnvironmentCubemap,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_construct) environment_brdf_lut:
        SceneEnvironmentBrdfLut,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_construct) bind_group:
        wgpu::BindGroup,
}
