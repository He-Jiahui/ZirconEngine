use crate::core::framework::render::RenderImageUsage;
use crate::core::math::Vec4;
use crate::core::resource::{MaterialMarker, ResourceHandle, ResourceId};
use crate::graphics::scene::resources::{
    MaterialDisabledPasses, MaterialRuntime, ResourceStreamer,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::{
    MaterialTextureBinding, MaterialTextureSet,
};

pub(super) fn material_tinted(
    streamer: &ResourceStreamer,
    material: ResourceHandle<MaterialMarker>,
    instance_tint: Vec4,
) -> Vec4 {
    let material_tint = streamer
        .material(&material.id())
        .map(|material| material.base_color)
        .unwrap_or(Vec4::ONE);
    instance_tint * material_tint
}

pub(super) fn material_receive_shadows(
    streamer: &ResourceStreamer,
    material: ResourceHandle<MaterialMarker>,
) -> bool {
    streamer
        .material(&material.id())
        .map(|material| material.receive_shadows)
        .unwrap_or(true)
}

pub(super) fn material_cast_shadows(
    streamer: &ResourceStreamer,
    material: ResourceHandle<MaterialMarker>,
) -> bool {
    streamer
        .material(&material.id())
        .map(|material| material.cast_shadows)
        .unwrap_or(true)
}

pub(super) fn material_taa_reactive_mask_strength(material: Option<&MaterialRuntime>) -> f32 {
    material
        .map(|material| material.taa_reactive_mask_strength)
        .filter(|strength| strength.is_finite())
        .unwrap_or_default()
        .clamp(0.0, 1.0)
}

pub(super) fn material_disabled_passes(
    material: Option<&MaterialRuntime>,
) -> MaterialDisabledPasses {
    material
        .map(|material| material.disabled_passes)
        .unwrap_or_default()
}

pub(super) fn material_texture_set(
    streamer: &ResourceStreamer,
    material: Option<&MaterialRuntime>,
) -> MaterialTextureSet {
    MaterialTextureSet::new(
        material_texture_binding(
            streamer,
            material.and_then(|material| material.base_color_texture),
        ),
        material_normal_texture_binding(
            streamer,
            material.and_then(|material| material.normal_texture),
        ),
        material_texture_binding(
            streamer,
            material.and_then(|material| material.metallic_roughness_texture),
        ),
        material_texture_binding(
            streamer,
            material.and_then(|material| material.occlusion_texture),
        ),
        material_texture_binding(
            streamer,
            material.and_then(|material| material.emissive_texture),
        ),
        material_normal_texture_binding(
            streamer,
            material.and_then(|material| material.clearcoat_normal_texture),
        ),
    )
}

fn material_texture_binding(
    streamer: &ResourceStreamer,
    texture_id: Option<ResourceId>,
) -> MaterialTextureBinding {
    material_output_target_texture_binding(streamer, texture_id)
        .unwrap_or_else(|| MaterialTextureBinding::texture(streamer.texture(texture_id)))
}

fn material_normal_texture_binding(
    streamer: &ResourceStreamer,
    texture_id: Option<ResourceId>,
) -> MaterialTextureBinding {
    material_output_target_texture_binding(streamer, texture_id)
        .unwrap_or_else(|| MaterialTextureBinding::texture(streamer.normal_texture(texture_id)))
}

fn material_output_target_texture_binding(
    streamer: &ResourceStreamer,
    texture_id: Option<ResourceId>,
) -> Option<MaterialTextureBinding> {
    let output_target = streamer.output_target_texture_resource(&texture_id?)?;
    output_target
        .descriptor()
        .usage
        .contains(&RenderImageUsage::Sampled)
        .then(|| MaterialTextureBinding::output_target(output_target))
}
