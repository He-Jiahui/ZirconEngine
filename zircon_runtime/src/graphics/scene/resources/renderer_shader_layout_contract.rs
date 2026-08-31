use crate::core::framework::render::{RenderShaderBindingResourceType, RenderShaderStage};
use crate::graphics::scene::gpu_scene::{
    GPU_SCENE_INSTANCE_DATA_BINDING, GPU_SCENE_LIGHT_DATA_BINDING,
    GPU_SCENE_PREVIOUS_SKINNED_JOINT_PALETTE_BINDING, GPU_SCENE_PRIMITIVE_DATA_BINDING,
    GPU_SCENE_SKINNED_JOINT_PALETTE_BINDING,
};

pub(in crate::graphics::scene) const MATERIAL_BIND_GROUP: u32 = 2;
pub(in crate::graphics::scene) const MATERIAL_UNIFORM_BINDING: u32 = 0;
pub(in crate::graphics::scene) const MATERIAL_BASE_COLOR_TEXTURE_BINDING: u32 = 1;
pub(in crate::graphics::scene) const MATERIAL_BASE_COLOR_SAMPLER_BINDING: u32 = 2;
pub(in crate::graphics::scene) const MATERIAL_NORMAL_TEXTURE_BINDING: u32 = 3;
pub(in crate::graphics::scene) const MATERIAL_NORMAL_SAMPLER_BINDING: u32 = 4;
pub(in crate::graphics::scene) const MATERIAL_METALLIC_ROUGHNESS_TEXTURE_BINDING: u32 = 5;
pub(in crate::graphics::scene) const MATERIAL_METALLIC_ROUGHNESS_SAMPLER_BINDING: u32 = 6;
pub(in crate::graphics::scene) const MATERIAL_OCCLUSION_TEXTURE_BINDING: u32 = 7;
pub(in crate::graphics::scene) const MATERIAL_OCCLUSION_SAMPLER_BINDING: u32 = 8;
pub(in crate::graphics::scene) const MATERIAL_EMISSIVE_TEXTURE_BINDING: u32 = 9;
pub(in crate::graphics::scene) const MATERIAL_EMISSIVE_SAMPLER_BINDING: u32 = 10;
pub(in crate::graphics::scene) const MATERIAL_CLEARCOAT_NORMAL_TEXTURE_BINDING: u32 = 11;
pub(in crate::graphics::scene) const MATERIAL_CLEARCOAT_NORMAL_SAMPLER_BINDING: u32 = 12;
pub(in crate::graphics::scene) const MATERIAL_BINDING_COUNT: usize = 13;
pub(in crate::graphics::scene) const GPU_SCENE_DRAW_BIND_GROUP: u32 = 3;
const GPU_SCENE_DRAW_BINDING_COUNT: usize = 5;

const MATERIAL_VERTEX_FRAGMENT_VISIBILITY: &[RenderShaderStage] =
    &[RenderShaderStage::Vertex, RenderShaderStage::Fragment];
const MATERIAL_FRAGMENT_VISIBILITY: &[RenderShaderStage] = &[RenderShaderStage::Fragment];
const GPU_SCENE_VERTEX_FRAGMENT_VISIBILITY: &[RenderShaderStage] =
    &[RenderShaderStage::Vertex, RenderShaderStage::Fragment];
const GPU_SCENE_VERTEX_VISIBILITY: &[RenderShaderStage] = &[RenderShaderStage::Vertex];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene) struct RendererShaderBindingContract {
    pub(in crate::graphics::scene) binding: u32,
    pub(in crate::graphics::scene) label: &'static str,
    pub(in crate::graphics::scene) resource_type: RenderShaderBindingResourceType,
    pub(in crate::graphics::scene) allowed_visibility: &'static [RenderShaderStage],
}

impl RendererShaderBindingContract {
    const fn new(
        binding: u32,
        label: &'static str,
        resource_type: RenderShaderBindingResourceType,
        allowed_visibility: &'static [RenderShaderStage],
    ) -> Self {
        Self {
            binding,
            label,
            resource_type,
            allowed_visibility,
        }
    }
}

const MATERIAL_SHADER_BINDING_CONTRACT: [RendererShaderBindingContract; MATERIAL_BINDING_COUNT] = [
    RendererShaderBindingContract::new(
        MATERIAL_UNIFORM_BINDING,
        "material property uniform",
        RenderShaderBindingResourceType::UniformBuffer,
        MATERIAL_VERTEX_FRAGMENT_VISIBILITY,
    ),
    RendererShaderBindingContract::new(
        MATERIAL_BASE_COLOR_TEXTURE_BINDING,
        "base-color texture",
        RenderShaderBindingResourceType::Texture,
        MATERIAL_FRAGMENT_VISIBILITY,
    ),
    RendererShaderBindingContract::new(
        MATERIAL_BASE_COLOR_SAMPLER_BINDING,
        "base-color sampler",
        RenderShaderBindingResourceType::Sampler,
        MATERIAL_FRAGMENT_VISIBILITY,
    ),
    RendererShaderBindingContract::new(
        MATERIAL_NORMAL_TEXTURE_BINDING,
        "normal texture",
        RenderShaderBindingResourceType::Texture,
        MATERIAL_FRAGMENT_VISIBILITY,
    ),
    RendererShaderBindingContract::new(
        MATERIAL_NORMAL_SAMPLER_BINDING,
        "normal sampler",
        RenderShaderBindingResourceType::Sampler,
        MATERIAL_FRAGMENT_VISIBILITY,
    ),
    RendererShaderBindingContract::new(
        MATERIAL_METALLIC_ROUGHNESS_TEXTURE_BINDING,
        "metallic-roughness texture",
        RenderShaderBindingResourceType::Texture,
        MATERIAL_FRAGMENT_VISIBILITY,
    ),
    RendererShaderBindingContract::new(
        MATERIAL_METALLIC_ROUGHNESS_SAMPLER_BINDING,
        "metallic-roughness sampler",
        RenderShaderBindingResourceType::Sampler,
        MATERIAL_FRAGMENT_VISIBILITY,
    ),
    RendererShaderBindingContract::new(
        MATERIAL_OCCLUSION_TEXTURE_BINDING,
        "occlusion texture",
        RenderShaderBindingResourceType::Texture,
        MATERIAL_FRAGMENT_VISIBILITY,
    ),
    RendererShaderBindingContract::new(
        MATERIAL_OCCLUSION_SAMPLER_BINDING,
        "occlusion sampler",
        RenderShaderBindingResourceType::Sampler,
        MATERIAL_FRAGMENT_VISIBILITY,
    ),
    RendererShaderBindingContract::new(
        MATERIAL_EMISSIVE_TEXTURE_BINDING,
        "emissive texture",
        RenderShaderBindingResourceType::Texture,
        MATERIAL_FRAGMENT_VISIBILITY,
    ),
    RendererShaderBindingContract::new(
        MATERIAL_EMISSIVE_SAMPLER_BINDING,
        "emissive sampler",
        RenderShaderBindingResourceType::Sampler,
        MATERIAL_FRAGMENT_VISIBILITY,
    ),
    RendererShaderBindingContract::new(
        MATERIAL_CLEARCOAT_NORMAL_TEXTURE_BINDING,
        "clearcoat-normal texture",
        RenderShaderBindingResourceType::Texture,
        MATERIAL_FRAGMENT_VISIBILITY,
    ),
    RendererShaderBindingContract::new(
        MATERIAL_CLEARCOAT_NORMAL_SAMPLER_BINDING,
        "clearcoat-normal sampler",
        RenderShaderBindingResourceType::Sampler,
        MATERIAL_FRAGMENT_VISIBILITY,
    ),
];

const GPU_SCENE_SHADER_BINDING_CONTRACT: [RendererShaderBindingContract;
    GPU_SCENE_DRAW_BINDING_COUNT] = [
    RendererShaderBindingContract::new(
        GPU_SCENE_PRIMITIVE_DATA_BINDING,
        "GPUScene primitive storage buffer",
        RenderShaderBindingResourceType::StorageBuffer,
        GPU_SCENE_VERTEX_FRAGMENT_VISIBILITY,
    ),
    RendererShaderBindingContract::new(
        GPU_SCENE_INSTANCE_DATA_BINDING,
        "GPUScene instance storage buffer",
        RenderShaderBindingResourceType::StorageBuffer,
        GPU_SCENE_VERTEX_FRAGMENT_VISIBILITY,
    ),
    RendererShaderBindingContract::new(
        GPU_SCENE_LIGHT_DATA_BINDING,
        "GPUScene light storage buffer",
        RenderShaderBindingResourceType::StorageBuffer,
        GPU_SCENE_VERTEX_FRAGMENT_VISIBILITY,
    ),
    RendererShaderBindingContract::new(
        GPU_SCENE_SKINNED_JOINT_PALETTE_BINDING,
        "current skinned joint palette storage",
        RenderShaderBindingResourceType::StorageBuffer,
        GPU_SCENE_VERTEX_VISIBILITY,
    ),
    RendererShaderBindingContract::new(
        GPU_SCENE_PREVIOUS_SKINNED_JOINT_PALETTE_BINDING,
        "previous skinned joint palette storage",
        RenderShaderBindingResourceType::StorageBuffer,
        GPU_SCENE_VERTEX_VISIBILITY,
    ),
];

pub(in crate::graphics::scene) const fn material_shader_binding_contract()
-> &'static [RendererShaderBindingContract; MATERIAL_BINDING_COUNT] {
    &MATERIAL_SHADER_BINDING_CONTRACT
}

pub(in crate::graphics::scene) const fn gpu_scene_shader_binding_contract()
-> &'static [RendererShaderBindingContract; GPU_SCENE_DRAW_BINDING_COUNT] {
    &GPU_SCENE_SHADER_BINDING_CONTRACT
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn material_shader_binding_contract_has_one_row_per_fixed_binding() {
        let contract = material_shader_binding_contract();
        assert_eq!(contract.len(), MATERIAL_BINDING_COUNT);
        assert_eq!(
            contract
                .iter()
                .map(|binding| binding.binding)
                .collect::<HashSet<_>>()
                .len(),
            MATERIAL_BINDING_COUNT
        );
        assert!(
            contract
                .iter()
                .enumerate()
                .all(|(index, binding)| binding.binding == index as u32)
        );
    }

    #[test]
    fn gpu_scene_shader_binding_contract_matches_the_draw_facing_subset() {
        let contract = gpu_scene_shader_binding_contract();
        assert_eq!(GPU_SCENE_DRAW_BIND_GROUP, 3);
        assert_eq!(contract.len(), GPU_SCENE_DRAW_BINDING_COUNT);
        assert_eq!(
            contract
                .iter()
                .map(|binding| binding.binding)
                .collect::<Vec<_>>(),
            vec![
                GPU_SCENE_PRIMITIVE_DATA_BINDING,
                GPU_SCENE_INSTANCE_DATA_BINDING,
                GPU_SCENE_LIGHT_DATA_BINDING,
                GPU_SCENE_SKINNED_JOINT_PALETTE_BINDING,
                GPU_SCENE_PREVIOUS_SKINNED_JOINT_PALETTE_BINDING,
            ]
        );
        assert!(contract.iter().all(|binding| {
            binding.resource_type == RenderShaderBindingResourceType::StorageBuffer
        }));
        assert_eq!(contract[3].allowed_visibility, GPU_SCENE_VERTEX_VISIBILITY);
        assert_eq!(contract[4].allowed_visibility, GPU_SCENE_VERTEX_VISIBILITY);
    }
}
