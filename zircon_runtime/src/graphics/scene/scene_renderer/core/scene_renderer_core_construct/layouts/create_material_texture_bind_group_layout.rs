use crate::core::framework::render::{RenderShaderBindingResourceType, RenderShaderStage};
use crate::graphics::scene::resources::{
    material_shader_binding_contract, RendererShaderBindingContract, GPU_MATERIAL_UNIFORM_MIN_SIZE,
    MATERIAL_BINDING_COUNT,
};

pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_construct) fn create_material_texture_bind_group_layout(
    device: &wgpu::Device,
) -> wgpu::BindGroupLayout {
    let entries = material_texture_bind_group_layout_entries();
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-material-set-layout"),
        entries: &entries,
    })
}

pub(in crate::graphics::scene::scene_renderer) fn material_texture_bind_group_layout_entries(
) -> [wgpu::BindGroupLayoutEntry; MATERIAL_BINDING_COUNT] {
    let contract = material_shader_binding_contract();
    std::array::from_fn(|index| material_layout_entry(contract[index]))
}

fn material_layout_entry(contract: RendererShaderBindingContract) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding: contract.binding,
        visibility: wgpu_visibility(contract.allowed_visibility),
        ty: match contract.resource_type {
            RenderShaderBindingResourceType::UniformBuffer => wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(GPU_MATERIAL_UNIFORM_MIN_SIZE as u64),
            },
            RenderShaderBindingResourceType::Texture => wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
            RenderShaderBindingResourceType::Sampler => {
                wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)
            }
            RenderShaderBindingResourceType::StorageBuffer
            | RenderShaderBindingResourceType::StorageTexture => {
                unreachable!("material renderer contract contains an unsupported resource class")
            }
        },
        count: None,
    }
}

fn wgpu_visibility(stages: &[RenderShaderStage]) -> wgpu::ShaderStages {
    stages
        .iter()
        .fold(wgpu::ShaderStages::empty(), |visibility, stage| {
            visibility
                | match stage {
                    RenderShaderStage::Vertex => wgpu::ShaderStages::VERTEX,
                    RenderShaderStage::Fragment => wgpu::ShaderStages::FRAGMENT,
                    RenderShaderStage::Compute => wgpu::ShaderStages::COMPUTE,
                }
        })
}

#[cfg(test)]
#[path = "tests/create_material_texture_bind_group_layout.rs"]
mod tests;
