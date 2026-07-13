use super::super::super::super::environment::scene_bind_group_layout_entries;
use super::super::super::super::primitives::{SceneEnvironmentSh9, SceneUniform};
use super::super::super::scene_renderer_core::{SceneEnvironmentBrdfLut, SceneEnvironmentCubemap};
use super::scene_bind_group_bundle::SceneBindGroupBundle;

pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_construct) fn create_scene_bind_group_bundle(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
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
    let environment_cubemap = SceneEnvironmentCubemap::fallback(device, queue);
    let environment_brdf_lut = SceneEnvironmentBrdfLut::new(device, queue);
    let environment_sh9_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-scene-environment-sh9"),
        size: SceneEnvironmentSh9::byte_len(),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(
        &environment_sh9_buffer,
        0,
        bytemuck::bytes_of(&SceneEnvironmentSh9::default()),
    );
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
