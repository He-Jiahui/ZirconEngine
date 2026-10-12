use super::*;
use crate::graphics::shader::{ShaderTextureSampleType, ShaderTextureViewDimension};

#[test]
fn required_resources_are_a_subset_of_the_pipeline_layout() {
    let contract = MeshShaderPipelineLayoutContract::try_new([
        layout_binding(
            0,
            0,
            ShaderBindingResourceType::UniformBuffer,
            &[ShaderBindingStage::Vertex, ShaderBindingStage::Fragment],
        ),
        layout_binding(
            0,
            1,
            sampled_texture(ShaderTextureViewDimension::D2),
            &[ShaderBindingStage::Fragment],
        ),
    ])
    .unwrap();

    assert!(contract
        .validate_requirements(&[requirement(
            0,
            0,
            ShaderBindingResourceType::UniformBuffer,
            ShaderBindingStage::Vertex,
        )])
        .is_ok());
}

#[test]
fn duplicate_and_missing_bindings_are_rejected() {
    let duplicate = layout_binding(
        2,
        4,
        ShaderBindingResourceType::Sampler { comparison: false },
        &[ShaderBindingStage::Fragment],
    );
    assert!(MeshShaderPipelineLayoutContract::try_new([duplicate, duplicate]).is_err());

    let contract = MeshShaderPipelineLayoutContract::try_new([]).unwrap();
    assert!(contract
        .validate_requirements(&[requirement(
            3,
            9,
            ShaderBindingResourceType::StorageBuffer { read_only: true },
            ShaderBindingStage::Vertex,
        )])
        .unwrap_err()
        .contains("group 3 binding 9"));
}

#[test]
fn resource_type_and_visibility_mismatches_are_rejected() {
    let contract = MeshShaderPipelineLayoutContract::try_new([
        layout_binding(
            1,
            2,
            ShaderBindingResourceType::Sampler { comparison: false },
            &[ShaderBindingStage::Fragment],
        ),
        layout_binding(
            2,
            0,
            ShaderBindingResourceType::UniformBuffer,
            &[ShaderBindingStage::Fragment],
        ),
    ])
    .unwrap();

    assert!(contract
        .validate_requirements(&[requirement(
            1,
            2,
            ShaderBindingResourceType::Sampler { comparison: true },
            ShaderBindingStage::Fragment,
        )])
        .unwrap_err()
        .contains("resource type mismatch"));
    assert!(contract
        .validate_requirements(&[requirement(
            2,
            0,
            ShaderBindingResourceType::UniformBuffer,
            ShaderBindingStage::Vertex,
        )])
        .unwrap_err()
        .contains("visibility"));
}

#[test]
fn texture_dimension_storage_access_and_unsupported_resources_are_exact() {
    let contract = MeshShaderPipelineLayoutContract::try_new([
        layout_binding(
            1,
            5,
            sampled_texture(ShaderTextureViewDimension::Cube),
            &[ShaderBindingStage::Fragment],
        ),
        layout_binding(
            3,
            1,
            ShaderBindingResourceType::StorageBuffer { read_only: true },
            &[ShaderBindingStage::Vertex],
        ),
        layout_binding(
            4,
            7,
            ShaderBindingResourceType::Unsupported,
            &[ShaderBindingStage::Fragment],
        ),
    ])
    .unwrap();

    assert!(contract
        .validate_requirements(&[requirement(
            1,
            5,
            sampled_texture(ShaderTextureViewDimension::D2),
            ShaderBindingStage::Fragment,
        )])
        .is_err());
    assert!(contract
        .validate_requirements(&[requirement(
            3,
            1,
            ShaderBindingResourceType::StorageBuffer { read_only: false },
            ShaderBindingStage::Vertex,
        )])
        .is_err());
    assert!(contract
        .validate_requirements(&[requirement(
            4,
            7,
            ShaderBindingResourceType::Unsupported,
            ShaderBindingStage::Fragment,
        )])
        .is_err());
}

#[test]
fn explicit_buffer_minimum_must_cover_the_shader_while_none_stays_late_bound() {
    let fixed = MeshShaderPipelineLayoutContract::try_new([layout_binding(
        0,
        0,
        ShaderBindingResourceType::UniformBuffer,
        &[ShaderBindingStage::Vertex],
    )
    .with_min_binding_size(Some(64))])
    .unwrap();
    let late_bound = MeshShaderPipelineLayoutContract::try_new([layout_binding(
        0,
        0,
        ShaderBindingResourceType::UniformBuffer,
        &[ShaderBindingStage::Vertex],
    )])
    .unwrap();
    let exact = requirement(
        0,
        0,
        ShaderBindingResourceType::UniformBuffer,
        ShaderBindingStage::Vertex,
    )
    .with_min_binding_size(Some(64));
    let too_large = exact.with_min_binding_size(Some(80));

    assert!(fixed.validate_requirement(exact).is_ok());
    assert!(fixed
        .validate_requirement(too_large)
        .unwrap_err()
        .contains("requires 80 bytes"));
    assert!(late_bound.validate_requirement(too_large).is_ok());
}

#[test]
fn sampling_pairs_reject_filtering_of_nonfilterable_float_textures() {
    let filtering = MeshShaderPipelineLayoutContract::try_new([
        layout_binding(
            2,
            3,
            sampled_texture(ShaderTextureViewDimension::D2),
            &[ShaderBindingStage::Fragment],
        )
        .with_texture_filterability(Some(false)),
        layout_binding(
            2,
            4,
            ShaderBindingResourceType::Sampler { comparison: false },
            &[ShaderBindingStage::Fragment],
        )
        .with_sampler_binding_type(Some(MeshShaderSamplerBindingType::Filtering)),
    ])
    .unwrap();
    let non_filtering = MeshShaderPipelineLayoutContract::try_new([
        layout_binding(
            2,
            3,
            sampled_texture(ShaderTextureViewDimension::D2),
            &[ShaderBindingStage::Fragment],
        )
        .with_texture_filterability(Some(false)),
        layout_binding(
            2,
            4,
            ShaderBindingResourceType::Sampler { comparison: false },
            &[ShaderBindingStage::Fragment],
        )
        .with_sampler_binding_type(Some(MeshShaderSamplerBindingType::NonFiltering)),
    ])
    .unwrap();
    let pair = MeshShaderSamplingPairRequirement::new(2, 3, 2, 4);

    assert!(filtering
        .validate_sampling_pair(pair)
        .unwrap_err()
        .contains("non-filterable float texture"));
    assert!(non_filtering.validate_sampling_pair(pair).is_ok());
}

fn sampled_texture(view_dimension: ShaderTextureViewDimension) -> ShaderBindingResourceType {
    ShaderBindingResourceType::SampledTexture {
        view_dimension,
        sample_type: ShaderTextureSampleType::Float,
        multisampled: false,
    }
}

fn layout_binding(
    group: u32,
    binding: u32,
    resource_type: ShaderBindingResourceType,
    stages: &[ShaderBindingStage],
) -> MeshShaderResourceLayoutBinding {
    MeshShaderResourceLayoutBinding::new(
        group,
        binding,
        resource_type,
        ShaderBindingVisibility::from_stages(stages.iter().copied()),
    )
}

fn requirement(
    group: u32,
    binding: u32,
    resource_type: ShaderBindingResourceType,
    stage: ShaderBindingStage,
) -> MeshShaderResourceRequirement {
    MeshShaderResourceRequirement::new(group, binding, resource_type, stage)
}
