use super::super::super::super::primitives::SceneUniform;
use super::scene_bind_group_bundle::SceneBindGroupBundle;

pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_new) fn create_scene_bind_group_bundle(
    device: &wgpu::Device,
) -> SceneBindGroupBundle {
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-scene-layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-scene-uniform"),
        size: std::mem::size_of::<SceneUniform>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-scene-bind-group"),
        layout: &layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });

    SceneBindGroupBundle {
        layout,
        uniform_buffer,
        bind_group,
    }
}
