use super::super::super::depth_sampling_mode::PostProcessDepthSamplingMode;

pub(crate) fn taa_resolve(
    device: &wgpu::Device,
    depth_sampling_mode: PostProcessDepthSamplingMode,
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-taa-resolve-bind-group-layout"),
        entries: &[
            texture_entry(0, wgpu::TextureSampleType::Float { filterable: false }),
            texture_entry(1, depth_sampling_mode.scene_depth_sample_type()),
            texture_entry(2, wgpu::TextureSampleType::Float { filterable: false }),
            texture_entry(3, wgpu::TextureSampleType::Float { filterable: false }),
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            texture_entry(5, wgpu::TextureSampleType::Float { filterable: false }),
        ],
    })
}

fn texture_entry(binding: u32, sample_type: wgpu::TextureSampleType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            multisampled: false,
            view_dimension: wgpu::TextureViewDimension::D2,
            sample_type,
        },
        count: None,
    }
}
