pub(in crate::graphics::scene::scene_renderer) fn scene_bind_group_layout_entries(
) -> [wgpu::BindGroupLayoutEntry; 6] {
    [
        scene_uniform_layout_entry(),
        environment_cube_texture_entry(1),
        environment_sampler_entry(2),
        environment_brdf_lut_entry(3),
        environment_cube_texture_entry(4),
        environment_cube_texture_entry(5),
    ]
}

fn scene_uniform_layout_entry() -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::VERTEX
            | wgpu::ShaderStages::FRAGMENT
            | wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn environment_cube_texture_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            multisampled: false,
            view_dimension: wgpu::TextureViewDimension::Cube,
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
        },
        count: None,
    }
}

fn environment_sampler_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    }
}

fn environment_brdf_lut_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
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

#[cfg(test)]
mod tests {
    use super::scene_bind_group_layout_entries;

    #[test]
    fn scene_bind_group_layout_includes_source_specular_and_irradiance_cubemaps() {
        let entries = scene_bind_group_layout_entries();

        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.binding)
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 4, 5]
        );
        assert_cube_texture(&entries[1]);
        assert_cube_texture(&entries[4]);
        assert_cube_texture(&entries[5]);
    }

    fn assert_cube_texture(entry: &wgpu::BindGroupLayoutEntry) {
        let wgpu::BindingType::Texture {
            multisampled: false,
            view_dimension: wgpu::TextureViewDimension::Cube,
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
        } = &entry.ty
        else {
            panic!(
                "entry {} should be a filterable cube texture",
                entry.binding
            );
        };
    }
}
