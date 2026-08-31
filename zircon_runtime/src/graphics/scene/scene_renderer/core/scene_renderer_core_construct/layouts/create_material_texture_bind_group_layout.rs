use crate::core::framework::render::{RenderShaderBindingResourceType, RenderShaderStage};
use crate::graphics::scene::resources::{
    GPU_MATERIAL_UNIFORM_MIN_SIZE, MATERIAL_BINDING_COUNT, RendererShaderBindingContract,
    material_shader_binding_contract,
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

pub(in crate::graphics::scene::scene_renderer) fn material_texture_bind_group_layout_entries()
-> [wgpu::BindGroupLayoutEntry; MATERIAL_BINDING_COUNT] {
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
mod tests {
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
}
