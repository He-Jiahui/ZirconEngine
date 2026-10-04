use crate::core::framework::render::{CastShadowsMode, RendererCommon};
use crate::core::math::Vec4;
use crate::graphics::scene::resources::{
    MaterialDisabledPasses, MaterialRuntime, PublishedMaterialDrawProxy,
    PublishedMaterialTextureBinding,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::{
    MaterialTextureBinding, MaterialTextureSet,
};

const ERROR_MATERIAL_BASE_COLOR: Vec4 = Vec4::new(1.0, 0.0, 1.0, 1.0);

pub(super) fn material_tinted(material: Option<&MaterialRuntime>, instance_tint: Vec4) -> Vec4 {
    resolve_material_tint(material.map(|material| material.base_color), instance_tint)
}

fn resolve_material_tint(material_tint: Option<Vec4>, instance_tint: Vec4) -> Vec4 {
    instance_tint * material_tint.unwrap_or(ERROR_MATERIAL_BASE_COLOR)
}

/// 合并 Renderer 的阴影意图与材质许可，随后交给待绘制项。
/// 材质禁止投影时，纯阴影 Renderer 也必须停用，否则没有可执行的可见 pass。
pub(super) fn renderer_common_for_material(
    common: &RendererCommon,
    material: Option<&MaterialRuntime>,
) -> RendererCommon {
    resolve_renderer_common_for_material(
        common,
        material
            .map(|material| material.cast_shadows)
            .unwrap_or(true),
        material
            .map(|material| material.receive_shadows)
            .unwrap_or(true),
    )
}

fn resolve_renderer_common_for_material(
    common: &RendererCommon,
    material_casts_shadows: bool,
    material_receives_shadows: bool,
) -> RendererCommon {
    let mut resolved = common.clone();
    if !material_casts_shadows {
        if resolved.cast_shadows == CastShadowsMode::ShadowsOnly {
            resolved.enabled = false;
        }
        resolved.cast_shadows = CastShadowsMode::Off;
    }
    resolved.receive_shadows &= material_receives_shadows;
    resolved
}

pub(super) fn material_taa_reactive_mask_strength(material: Option<&MaterialRuntime>) -> f32 {
    material
        .map(|material| material.taa_reactive_mask_strength)
        .filter(|strength| strength.is_finite())
        .unwrap_or_default()
        .clamp(0.0, 1.0)
}

pub(super) fn material_half_resolution_transparency(material: Option<&MaterialRuntime>) -> bool {
    material.is_some_and(|material| material.alpha_blend && material.separate_translucency)
}

pub(super) fn material_disabled_passes(
    material: Option<&MaterialRuntime>,
) -> MaterialDisabledPasses {
    material
        .map(|material| material.disabled_passes)
        .unwrap_or_default()
}

pub(super) fn material_texture_set(material: PublishedMaterialDrawProxy<'_>) -> MaterialTextureSet {
    let textures = material.textures();
    MaterialTextureSet::new(
        material_texture_binding(textures.base_color),
        material_texture_binding(textures.normal),
        material_texture_binding(textures.metallic_roughness),
        material_texture_binding(textures.occlusion),
        material_texture_binding(textures.emissive),
        material_texture_binding(textures.clearcoat_normal),
    )
}

fn material_texture_binding(binding: PublishedMaterialTextureBinding) -> MaterialTextureBinding {
    match binding {
        PublishedMaterialTextureBinding::Texture(resource) => {
            MaterialTextureBinding::texture(resource)
        }
        PublishedMaterialTextureBinding::OutputTarget(resource) => {
            MaterialTextureBinding::output_target(resource)
        }
    }
}

#[cfg(test)]
#[path = "tests/material_inputs.rs"]
mod tests;
