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
        if let Some(seed) = self.published_material_draw_proxy(id).capture_seed() {
            crate::profile_counter!("render", "material_capture_published_proxy", 1);
            crate::profile_counter!(
                "render",
                "material_capture_generation_bound_texture_samples",
                [
                    seed.base_color_texture_revision
                        .zip(seed.base_color_texture_center_rgba),
                    seed.normal_texture_revision
                        .zip(seed.normal_texture_center_rgba),
                    seed.metallic_roughness_texture_revision
                        .zip(seed.metallic_roughness_texture_center_rgba),
                    seed.occlusion_texture_revision
                        .zip(seed.occlusion_texture_center_rgba),
                    seed.emissive_texture_revision
                        .zip(seed.emissive_texture_center_rgba),
                ]
                .into_iter()
                .flatten()
                .count(),
            );
            return Some(seed);
        }
        if self.materials.contains_key(id) {
            crate::profile_counter!("render", "material_capture_unpublished_fallback", 1);
            return None;
        }
        crate::profile_counter!("render", "material_capture_cold_asset_resolution", 1);
        let asset_manager = self.asset_manager().ok()?;
        let (material, _) = asset_manager.load_effective_material_asset(*id).ok()?;
        let shader_contract =
            Self::load_shader_contract(asset_manager.as_ref(), material.shader.clone());
        let descriptor = shader_contract
            .as_ref()
            .map(|shader| material.standard_material_descriptor_for_shader(shader.asset()))
            .unwrap_or_else(|| material.standard_material_descriptor());
        let lighting_model = if descriptor.unlit {
            RenderMaterialLightingModel::Unlit
        } else {
            descriptor.lighting_model.clone()
        };
        let shading_model_id = self.shading_model_id_for_lighting_model(&lighting_model);
        let base_color_texture = self
            .resolve_texture_reference("base_color_texture", descriptor.base_color_texture.as_ref())
            .id();
        let normal_texture = self
            .resolve_texture_reference("normal_texture", descriptor.normal_texture.as_ref())
            .id();
        let metallic_roughness_texture = self
            .resolve_texture_reference(
                "metallic_roughness_texture",
                descriptor.metallic_roughness_texture.as_ref(),
            )
            .id();
        let occlusion_texture = self
            .resolve_texture_reference("occlusion_texture", descriptor.occlusion_texture.as_ref())
            .id();
        let emissive_texture = self
            .resolve_texture_reference("emissive_texture", descriptor.emissive_texture.as_ref())
            .id();
        let (base_color_texture_revision, base_color_texture_center_rgba) =
            self.current_texture_capture_snapshot(asset_manager.as_ref(), base_color_texture);
        let (normal_texture_revision, normal_texture_center_rgba) =
            self.current_texture_capture_snapshot(asset_manager.as_ref(), normal_texture);
        let (metallic_roughness_texture_revision, metallic_roughness_texture_center_rgba) = self
            .current_texture_capture_snapshot(asset_manager.as_ref(), metallic_roughness_texture);
        let (occlusion_texture_revision, occlusion_texture_center_rgba) =
            self.current_texture_capture_snapshot(asset_manager.as_ref(), occlusion_texture);
        let (emissive_texture_revision, emissive_texture_center_rgba) =
            self.current_texture_capture_snapshot(asset_manager.as_ref(), emissive_texture);
        Some(MaterialCaptureSeed {
            base_color: Vec4::from_array(descriptor.base_color),
            emissive: Vec3::from_array(descriptor.emissive),
            metallic: descriptor.metallic,
            roughness: descriptor.roughness,
            occlusion_strength: descriptor.occlusion_strength,
            normal_scale: descriptor.normal_scale,
            double_sided: descriptor.double_sided,
            alpha_blend: matches!(descriptor.alpha_mode, RenderMaterialAlphaMode::Blend),
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
            base_color_texture,
            base_color_texture_revision,
            base_color_texture_center_rgba,
            base_color_texture_transform: descriptor.base_color_texture_transform,
            base_color_texture_uv_channel: descriptor.base_color_texture_uv_channel,
            normal_texture,
            normal_texture_revision,
            normal_texture_center_rgba,
            normal_texture_transform: descriptor.normal_texture_transform,
            normal_texture_uv_channel: descriptor.normal_texture_uv_channel,
            metallic_roughness_texture,
            metallic_roughness_texture_revision,
            metallic_roughness_texture_center_rgba,
            metallic_roughness_texture_transform: descriptor.metallic_roughness_texture_transform,
            metallic_roughness_texture_uv_channel: descriptor.metallic_roughness_texture_uv_channel,
            occlusion_texture,
            occlusion_texture_revision,
            occlusion_texture_center_rgba,
            occlusion_texture_transform: descriptor.occlusion_texture_transform,
            occlusion_texture_uv_channel: descriptor.occlusion_texture_uv_channel,
            emissive_texture,
            emissive_texture_revision,
            emissive_texture_center_rgba,
            emissive_texture_transform: descriptor.emissive_texture_transform,
            emissive_texture_uv_channel: descriptor.emissive_texture_uv_channel,
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

    fn current_texture_capture_snapshot(
        &self,
        asset_manager: &crate::asset::ProjectAssetManager,
        id: Option<ResourceId>,
    ) -> (Option<u64>, Option<Vec4>) {
        let Some(id) = id else {
            return (None, None);
        };
        let Ok(texture) = asset_manager.load_texture_asset_snapshot(id) else {
            return (None, None);
        };
        let sample = sample_texture_asset_rgba(&texture, [0.5, 0.5]);
        (Some(texture.revision()), sample)
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

#[cfg(test)]
mod tests {
    #[test]
    fn capture_reads_only_published_runtime_state_before_cold_asset_fallback() {
        let production = include_str!("material_capture.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("material capture test boundary");
        let published = production
            .find("published_material_draw_proxy(id)")
            .expect("published draw proxy lookup");
        let cold_fallback = production
            .find("load_effective_material_asset")
            .expect("canonical cold material fallback");

        assert!(published < cold_fallback);
        assert!(production.contains("self.materials.contains_key(id)"));
        assert!(!production.contains("self.material(id)"));
        assert!(production.contains("material_capture_published_proxy"));
        assert!(production.contains("material_capture_generation_bound_texture_samples"));
    }

    #[test]
    fn cold_texture_capture_uses_one_generation_bound_snapshot() {
        let production = include_str!("material_capture.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("material capture test boundary");

        assert!(production.contains("load_texture_asset_snapshot(id)"));
        assert!(production.contains("Some(texture.revision())"));
        assert!(!production.contains("let Ok(revision) = self.resource_revision(id)"));
    }
}
