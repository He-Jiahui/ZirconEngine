use serde::{Deserialize, Serialize};

use super::{
    AdvancedPbrMaterialFrameUsage, FogVolumeData, FroxelGridQuality, IrradianceVolumeData,
    LightCookieData, OitSettings, PlanarReflectionProbeData, ScreenSpaceTransmissionSettings,
    SubsurfaceProfileData, VolumetricFogSettings,
};

/// Optional advanced-lighting frame sideband. Empty vectors and `None` keep
/// feature-disabled frames free of authored advanced-lighting payloads.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AdvancedLightingExtract {
    pub material_features: AdvancedPbrMaterialFrameUsage,
    #[serde(default)]
    pub screen_space_transmission: ScreenSpaceTransmissionSettings,
    pub volumetric: Option<VolumetricFogSettings>,
    pub oit: Option<OitSettings>,
    pub fog_volumes: Vec<FogVolumeData>,
    /// Stable ids of authored lights that participate in froxel scattering.
    pub volumetric_light_ids: Vec<u64>,
    pub cookies: Vec<LightCookieData>,
    pub irradiance_volumes: Vec<IrradianceVolumeData>,
    pub planar_probes: Vec<PlanarReflectionProbeData>,
    pub subsurface_profiles: Vec<SubsurfaceProfileData>,
    /// Profile ids referenced by Subsurface materials in the current view.
    /// An empty set keeps all SSS graph work out of material-free views.
    pub subsurface_material_profile_indices: Vec<u32>,
}

impl AdvancedLightingExtract {
    pub fn is_empty(&self) -> bool {
        self.material_features.is_empty()
            && self.volumetric.is_none()
            && self.oit.is_none()
            && self.fog_volumes.is_empty()
            && self.volumetric_light_ids.is_empty()
            && self.cookies.is_empty()
            && self.irradiance_volumes.is_empty()
            && self.planar_probes.is_empty()
            && self.subsurface_profiles.is_empty()
            && self.subsurface_material_profile_indices.is_empty()
    }

    pub fn uses_subsurface_profile(&self, profile_id: u32) -> bool {
        self.subsurface_material_profile_indices
            .contains(&profile_id)
    }

    pub const fn froxel_dimensions(&self, quality: FroxelGridQuality) -> [u32; 3] {
        quality.dimensions()
    }

    pub fn fog_volumes_for_layers(
        &self,
        render_layers: &crate::core::framework::render::RenderLayerSet,
    ) -> Vec<FogVolumeData> {
        self.fog_volumes
            .iter()
            .filter(|volume| volume.layer_mask.intersects(render_layers))
            .cloned()
            .collect()
    }

    pub fn light_participates_in_volumetrics(&self, light_id: u64) -> bool {
        self.volumetric_light_ids.contains(&light_id)
    }

    pub const fn transmission_scene_copy_step_count(&self) -> usize {
        if self.material_features.requires_scene_color_copy() {
            self.screen_space_transmission.steps()
        } else {
            0
        }
    }

    pub const fn requires_transmission_scene_copy(&self) -> bool {
        self.transmission_scene_copy_step_count() > 0
    }

    pub const fn transmission_draw_step_count(&self) -> usize {
        if !self.material_features.uses_transmission() {
            0
        } else {
            let copy_steps = self.transmission_scene_copy_step_count();
            if copy_steps == 0 {
                1
            } else {
                copy_steps
            }
        }
    }
}

#[cfg(test)]
mod tests;
