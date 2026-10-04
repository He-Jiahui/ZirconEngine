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
#[path = "tests/mesh_shader_resource_contract_wgpu.rs"]
mod tests;
