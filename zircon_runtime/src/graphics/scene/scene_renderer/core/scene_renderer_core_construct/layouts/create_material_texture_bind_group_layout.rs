use crate::graphics::scene::resources::GPU_MATERIAL_UNIFORM_MIN_SIZE;

pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_construct) fn create_material_texture_bind_group_layout(
    device: &wgpu::Device,
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-material-set-layout"),
        entries: &[
            material_uniform_entry(0),
            material_texture_entry(1),
            material_sampler_entry(2),
            material_texture_entry(3),
            material_sampler_entry(4),
            material_texture_entry(5),
            material_sampler_entry(6),
            material_texture_entry(7),
            material_sampler_entry(8),
            material_texture_entry(9),
            material_sampler_entry(10),
        ],
    })
}

fn material_texture_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            multisampled: false,
            view_dimension: wgpu::TextureViewDimension::D2,
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
        },
        count: None,
    }
}

fn material_sampler_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    }
}

fn material_uniform_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: wgpu::BufferSize::new(GPU_MATERIAL_UNIFORM_MIN_SIZE as u64),
        },
        count: None,
    }
}
