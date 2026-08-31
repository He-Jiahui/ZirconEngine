use super::super::super::super::environment::scene_bind_group_layout_entries;
use super::super::super::super::primitives::{SceneEnvironmentSh9, SceneUniform};
use super::super::super::scene_renderer_core::{SceneEnvironmentBrdfLut, SceneEnvironmentCubemap};
use super::scene_bind_group_bundle::SceneBindGroupBundle;
use crate::graphics::backend::SystemTextureGenerationLease;
use wgpu::util::DeviceExt;

pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_construct) fn create_scene_bind_group_bundle(
    device: &wgpu::Device,
    system_textures: &SystemTextureGenerationLease,
) -> SceneBindGroupBundle {
    let scene_layout_entries = scene_bind_group_layout_entries();
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-scene-layout"),
        entries: &scene_layout_entries,
    });
    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-scene-uniform"),
        size: std::mem::size_of::<SceneUniform>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let environment_cubemap = SceneEnvironmentCubemap::fallback(system_textures);
    let environment_brdf_lut = SceneEnvironmentBrdfLut::from_system_textures(system_textures);
    let environment_sh9 = SceneEnvironmentSh9::default();
    let environment_sh9_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-scene-environment-sh9"),
        contents: bytemuck::bytes_of(&environment_sh9),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-scene-bind-group"),
        layout: &layout,
        entries: &environment_cubemap.bind_group_entries(
            &uniform_buffer,
            &environment_brdf_lut,
            &environment_sh9_buffer,
        ),
    });

    SceneBindGroupBundle {
        layout,
        uniform_buffer,
        environment_sh9_buffer,
        environment_cubemap,
        environment_brdf_lut,
        bind_group,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn scene_sh9_default_uses_cold_mapped_initialization() {
        let production = include_str!("create_scene_bind_group_bundle.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("scene bind group construction must retain a test boundary");

        assert!(production.contains("device.create_buffer_init("));
        assert!(production.contains("contents: bytemuck::bytes_of(&environment_sh9)"));
        assert!(production.contains("wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST"));
        assert!(!production.contains("queue.write_buffer("));
    }

    #[test]
    fn scene_bundle_consumes_system_textures_without_queue_authority() {
        let production = include_str!("create_scene_bind_group_bundle.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("scene bind group construction must retain a test boundary");

        assert!(production.contains("SceneEnvironmentCubemap::fallback(system_textures)"));
        assert!(
            production.contains("SceneEnvironmentBrdfLut::from_system_textures(system_textures)")
        );
        assert!(!production.contains("queue: &wgpu::Queue"));
        assert!(!production.contains("queue.write_texture"));
    }
}
