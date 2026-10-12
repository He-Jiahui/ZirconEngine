use super::super::mesh_shader_resource_contract::{
    MeshShaderResourceRequirement, MeshShaderSamplingPairRequirement,
};
use super::*;

#[test]
fn wgpu_entries_preserve_shader_visible_resource_semantics() {
    let entries = [
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Depth,
                view_dimension: wgpu::TextureViewDimension::CubeArray,
                multisampled: false,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 3,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(64),
            },
            count: None,
        },
    ];
    let contract =
        MeshShaderPipelineLayoutContract::from_wgpu_bind_group_layouts([(6, entries.as_slice())])
            .unwrap();

    assert!(contract
        .validate_requirements(&[
            MeshShaderResourceRequirement::new(
                6,
                0,
                ShaderBindingResourceType::StorageBuffer { read_only: true },
                ShaderBindingStage::Vertex,
            ),
            MeshShaderResourceRequirement::new(
                6,
                1,
                ShaderBindingResourceType::SampledTexture {
                    view_dimension: ShaderTextureViewDimension::CubeArray,
                    sample_type: ShaderTextureSampleType::Depth,
                    multisampled: false,
                },
                ShaderBindingStage::Fragment,
            ),
            MeshShaderResourceRequirement::new(
                6,
                2,
                ShaderBindingResourceType::Sampler { comparison: true },
                ShaderBindingStage::Fragment,
            ),
            MeshShaderResourceRequirement::new(
                6,
                3,
                ShaderBindingResourceType::UniformBuffer,
                ShaderBindingStage::Vertex,
            )
            .with_min_binding_size(Some(64)),
        ])
        .is_ok());
    assert!(contract
        .validate_requirement(
            MeshShaderResourceRequirement::new(
                6,
                3,
                ShaderBindingResourceType::UniformBuffer,
                ShaderBindingStage::Vertex,
            )
            .with_min_binding_size(Some(80))
        )
        .unwrap_err()
        .contains("requires 80 bytes"));
}

#[test]
fn wgpu_entries_preserve_sampler_operation_and_float_filterability() {
    let texture = wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    };
    let filtering_sampler = wgpu::BindGroupLayoutEntry {
        binding: 1,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    };
    let non_filtering_sampler = wgpu::BindGroupLayoutEntry {
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
        ..filtering_sampler
    };
    let pair = MeshShaderSamplingPairRequirement::new(5, 0, 5, 1);

    let filtering = MeshShaderPipelineLayoutContract::from_wgpu_bind_group_layouts([(
        5,
        [texture, filtering_sampler].as_slice(),
    )])
    .unwrap();
    assert!(filtering.validate_sampling_pair(pair).is_err());

    let non_filtering = MeshShaderPipelineLayoutContract::from_wgpu_bind_group_layouts([(
        5,
        [texture, non_filtering_sampler].as_slice(),
    )])
    .unwrap();
    assert!(non_filtering.validate_sampling_pair(pair).is_ok());
}
