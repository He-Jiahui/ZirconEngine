use crate::graphics::shader::{
    ShaderBindingResourceType, ShaderBindingStage, ShaderBindingVisibility,
    ShaderTextureSampleType, ShaderTextureViewDimension,
};

use super::mesh_shader_resource_contract::{
    MeshShaderPipelineLayoutContract, MeshShaderResourceLayoutBinding, MeshShaderSamplerBindingType,
};

impl MeshShaderPipelineLayoutContract {
    pub(super) fn from_wgpu_bind_group_layouts<'a>(
        bind_groups: impl IntoIterator<Item = (u32, &'a [wgpu::BindGroupLayoutEntry])>,
    ) -> Result<Self, String> {
        Self::try_new(bind_groups.into_iter().flat_map(|(group, entries)| {
            entries.iter().map(move |entry| {
                MeshShaderResourceLayoutBinding::new(
                    group,
                    entry.binding,
                    wgpu_binding_resource_type(entry),
                    wgpu_binding_visibility(entry.visibility),
                )
                .with_min_binding_size(wgpu_min_binding_size(entry))
                .with_texture_filterability(wgpu_texture_filterability(entry))
                .with_sampler_binding_type(wgpu_sampler_binding_type(entry))
            })
        }))
    }
}

fn wgpu_texture_filterability(entry: &wgpu::BindGroupLayoutEntry) -> Option<bool> {
    match entry.ty {
        wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable },
            ..
        } => Some(filterable),
        _ => None,
    }
}

fn wgpu_sampler_binding_type(
    entry: &wgpu::BindGroupLayoutEntry,
) -> Option<MeshShaderSamplerBindingType> {
    match entry.ty {
        wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering) => {
            Some(MeshShaderSamplerBindingType::Filtering)
        }
        wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering) => {
            Some(MeshShaderSamplerBindingType::NonFiltering)
        }
        wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison) => {
            Some(MeshShaderSamplerBindingType::Comparison)
        }
        _ => None,
    }
}

fn wgpu_min_binding_size(entry: &wgpu::BindGroupLayoutEntry) -> Option<u64> {
    match entry.ty {
        wgpu::BindingType::Buffer {
            min_binding_size, ..
        } => min_binding_size.map(wgpu::BufferSize::get),
        _ => None,
    }
}

fn wgpu_binding_resource_type(entry: &wgpu::BindGroupLayoutEntry) -> ShaderBindingResourceType {
    if entry.count.is_some() {
        return ShaderBindingResourceType::Unsupported;
    }
    match entry.ty {
        wgpu::BindingType::Buffer { ty, .. } => match ty {
            wgpu::BufferBindingType::Uniform => ShaderBindingResourceType::UniformBuffer,
            wgpu::BufferBindingType::Storage { read_only } => {
                ShaderBindingResourceType::StorageBuffer { read_only }
            }
        },
        wgpu::BindingType::Sampler(binding_type) => ShaderBindingResourceType::Sampler {
            comparison: binding_type == wgpu::SamplerBindingType::Comparison,
        },
        wgpu::BindingType::Texture {
            sample_type,
            view_dimension,
            multisampled,
        } => ShaderBindingResourceType::SampledTexture {
            view_dimension: wgpu_texture_view_dimension(view_dimension),
            sample_type: wgpu_texture_sample_type(sample_type),
            multisampled,
        },
        wgpu::BindingType::StorageTexture { .. }
        | wgpu::BindingType::AccelerationStructure { .. }
        | wgpu::BindingType::ExternalTexture => ShaderBindingResourceType::Unsupported,
    }
}

fn wgpu_binding_visibility(stages: wgpu::ShaderStages) -> ShaderBindingVisibility {
    let candidates = [
        (wgpu::ShaderStages::VERTEX, ShaderBindingStage::Vertex),
        (wgpu::ShaderStages::TASK, ShaderBindingStage::Task),
        (wgpu::ShaderStages::MESH, ShaderBindingStage::Mesh),
        (wgpu::ShaderStages::FRAGMENT, ShaderBindingStage::Fragment),
        (wgpu::ShaderStages::COMPUTE, ShaderBindingStage::Compute),
        (
            wgpu::ShaderStages::RAY_GENERATION,
            ShaderBindingStage::RayGeneration,
        ),
        (wgpu::ShaderStages::MISS, ShaderBindingStage::Miss),
        (wgpu::ShaderStages::ANY_HIT, ShaderBindingStage::AnyHit),
        (
            wgpu::ShaderStages::CLOSEST_HIT,
            ShaderBindingStage::ClosestHit,
        ),
    ];
    ShaderBindingVisibility::from_stages(
        candidates
            .into_iter()
            .filter_map(|(flag, stage)| stages.contains(flag).then_some(stage)),
    )
}

const fn wgpu_texture_view_dimension(
    dimension: wgpu::TextureViewDimension,
) -> ShaderTextureViewDimension {
    match dimension {
        wgpu::TextureViewDimension::D1 => ShaderTextureViewDimension::D1,
        wgpu::TextureViewDimension::D2 => ShaderTextureViewDimension::D2,
        wgpu::TextureViewDimension::D2Array => ShaderTextureViewDimension::D2Array,
        wgpu::TextureViewDimension::Cube => ShaderTextureViewDimension::Cube,
        wgpu::TextureViewDimension::CubeArray => ShaderTextureViewDimension::CubeArray,
        wgpu::TextureViewDimension::D3 => ShaderTextureViewDimension::D3,
    }
}

const fn wgpu_texture_sample_type(sample_type: wgpu::TextureSampleType) -> ShaderTextureSampleType {
    match sample_type {
        wgpu::TextureSampleType::Float { .. } => ShaderTextureSampleType::Float,
        wgpu::TextureSampleType::Depth => ShaderTextureSampleType::Depth,
        wgpu::TextureSampleType::Sint => ShaderTextureSampleType::Sint,
        wgpu::TextureSampleType::Uint => ShaderTextureSampleType::Uint,
    }
}

#[cfg(test)]
mod tests {
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
        let contract = MeshShaderPipelineLayoutContract::from_wgpu_bind_group_layouts([(
            6,
            entries.as_slice(),
        )])
        .unwrap();

        assert!(
            contract
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
                .is_ok()
        );
        assert!(
            contract
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
                .contains("requires 80 bytes")
        );
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
}
