use crate::asset::TextureAsset;
use crate::core::framework::render::{
    RenderMaterialAlphaMode, RenderMaterialLightingModel, SHADING_MODEL_ID_STANDARD_PBR,
    ShadingModelId,
};
use crate::core::math::{Vec3, Vec4};
use crate::core::resource::ResourceId;

use super::super::super::MaterialCaptureSeed;
use super::super::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn material_capture_seed(&self, id: &ResourceId) -> Option<MaterialCaptureSeed> {
        self.material(id)
            .map(|material| material.capture_seed())
            .or_else(|| {
                self.asset_manager()
                    .ok()?
                    .load_material_asset(*id)
                    .ok()
                    .map(|material| {
                        let descriptor = material.standard_material_descriptor();
                        let lighting_model = if descriptor.unlit {
                            RenderMaterialLightingModel::Unlit
                        } else {
                            descriptor.lighting_model.clone()
                        };
                        let shading_model_id =
                            self.shading_model_id_for_lighting_model(&lighting_model);
                        MaterialCaptureSeed {
                            base_color: Vec4::from_array(descriptor.base_color),
                            emissive: Vec3::from_array(descriptor.emissive),
                            metallic: descriptor.metallic,
                            roughness: descriptor.roughness,
                            double_sided: descriptor.double_sided,
                            alpha_blend: matches!(
                                descriptor.alpha_mode,
                                RenderMaterialAlphaMode::Blend
                            ),
                            alpha_cutoff: match descriptor.alpha_mode {
                                RenderMaterialAlphaMode::Mask { cutoff } => Some(cutoff),
                                _ => None,
                            },
                            lighting_model,
                            shading_model_id,
                            unlit: descriptor.unlit || descriptor.lighting_model.is_unlit(),
                            cast_shadows: descriptor.cast_shadows,
                            receive_shadows: descriptor.receive_shadows,
                            disabled_passes: Default::default(),
                            taa_reactive_mask_strength: descriptor.taa_reactive_mask_strength,
                            subsurface_profile_index: descriptor.subsurface_profile_index,
                            base_color_texture: self
                                .resolve_texture_reference(
                                    "base_color_texture",
                                    descriptor.base_color_texture.as_ref(),
                                )
                                .id(),
                            base_color_texture_transform: descriptor.base_color_texture_transform,
                            base_color_texture_uv_channel: descriptor.base_color_texture_uv_channel,
                            normal_texture: self
                                .resolve_texture_reference(
                                    "normal_texture",
                                    descriptor.normal_texture.as_ref(),
                                )
                                .id(),
                            normal_texture_transform: descriptor.normal_texture_transform,
                            normal_texture_uv_channel: descriptor.normal_texture_uv_channel,
                            metallic_roughness_texture: self
                                .resolve_texture_reference(
                                    "metallic_roughness_texture",
                                    descriptor.metallic_roughness_texture.as_ref(),
                                )
                                .id(),
                            metallic_roughness_texture_transform: descriptor
                                .metallic_roughness_texture_transform,
                            metallic_roughness_texture_uv_channel: descriptor
                                .metallic_roughness_texture_uv_channel,
                            occlusion_texture: self
                                .resolve_texture_reference(
                                    "occlusion_texture",
                                    descriptor.occlusion_texture.as_ref(),
                                )
                                .id(),
                            occlusion_texture_transform: descriptor.occlusion_texture_transform,
                            occlusion_texture_uv_channel: descriptor.occlusion_texture_uv_channel,
                            emissive_texture: self
                                .resolve_texture_reference(
                                    "emissive_texture",
                                    descriptor.emissive_texture.as_ref(),
                                )
                                .id(),
                            emissive_texture_transform: descriptor.emissive_texture_transform,
                            emissive_texture_uv_channel: descriptor.emissive_texture_uv_channel,
                        }
                    })
            })
    }

    pub(crate) fn sample_texture_rgba(&self, id: Option<ResourceId>, uv: [f32; 2]) -> Option<Vec4> {
        id.and_then(|texture_id| {
            self.asset_manager()
                .ok()?
                .load_texture_asset(texture_id)
                .ok()
                .and_then(|texture| sample_texture_asset_rgba(&texture, uv))
        })
    }

    fn shading_model_id_for_lighting_model(
        &self,
        model: &RenderMaterialLightingModel,
    ) -> ShadingModelId {
        self.shading_model_registry
            .resolve_lighting_model(model)
            .map(|descriptor| descriptor.id)
            .unwrap_or(SHADING_MODEL_ID_STANDARD_PBR)
    }
}

fn sample_texture_asset_rgba(texture: &TextureAsset, uv: [f32; 2]) -> Option<Vec4> {
    if texture.width == 0 || texture.height == 0 {
        return None;
    }

    let u = wrap01(uv[0]);
    let v = wrap01(uv[1]);
    let x = ((texture.width - 1) as f32 * u).round() as usize;
    let y = ((texture.height - 1) as f32 * v).round() as usize;
    let index = ((y * texture.width as usize) + x) * 4;
    let rgba = texture.rgba.get(index..index + 4)?;
    Some(Vec4::new(
        rgba[0] as f32 / 255.0,
        rgba[1] as f32 / 255.0,
        rgba[2] as f32 / 255.0,
        rgba[3] as f32 / 255.0,
    ))
}

fn wrap01(value: f32) -> f32 {
    let wrapped = value.fract();
    if wrapped < 0.0 {
        wrapped + 1.0
    } else {
        wrapped
    }
}
