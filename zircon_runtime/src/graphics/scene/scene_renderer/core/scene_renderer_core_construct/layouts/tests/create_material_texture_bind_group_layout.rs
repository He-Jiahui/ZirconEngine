use super::*;

#[test]
fn material_wgpu_layout_entries_project_the_canonical_contract() {
    let contract = material_shader_binding_contract();
    let entries = material_texture_bind_group_layout_entries();

    assert_eq!(entries.len(), contract.len());
    for (entry, expected) in entries.iter().zip(contract) {
        assert_eq!(entry.binding, expected.binding);
        assert_eq!(
            entry.visibility,
            wgpu_visibility(expected.allowed_visibility)
        );
        assert_eq!(entry.count, None);

        match (&entry.ty, expected.resource_type) {
            (
                wgpu::BindingType::Buffer {
                    ty,
                    has_dynamic_offset,
                    min_binding_size,
                },
                RenderShaderBindingResourceType::UniformBuffer,
            ) => {
                assert_eq!(*ty, wgpu::BufferBindingType::Uniform);
                assert!(!has_dynamic_offset);
                assert_eq!(
                    *min_binding_size,
                    wgpu::BufferSize::new(GPU_MATERIAL_UNIFORM_MIN_SIZE as u64)
                );
            }
            (
                wgpu::BindingType::Texture {
                    multisampled,
                    view_dimension,
                    sample_type,
                },
                RenderShaderBindingResourceType::Texture,
            ) => {
                assert!(!multisampled);
                assert_eq!(*view_dimension, wgpu::TextureViewDimension::D2);
                assert_eq!(
                    *sample_type,
                    wgpu::TextureSampleType::Float { filterable: true }
                );
            }
            (
                wgpu::BindingType::Sampler(sampler_type),
                RenderShaderBindingResourceType::Sampler,
            ) => assert_eq!(*sampler_type, wgpu::SamplerBindingType::Filtering),
            (actual, resource_type) => {
                panic!("unexpected WGPU projection for {resource_type:?}: {actual:?}")
            }
        }
    }
}
