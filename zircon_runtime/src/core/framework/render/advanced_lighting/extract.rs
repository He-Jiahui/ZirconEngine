use serde::{Deserialize, Serialize};

use super::{
    FogVolumeData, FroxelGridQuality, IrradianceVolumeData, LightCookieData, OitSettings,
    PlanarReflectionProbeData, SubsurfaceProfileData, VolumetricFogSettings,
};

/// Optional advanced-lighting frame sideband. Empty vectors and `None` keep
/// feature-disabled frames free of authored advanced-lighting payloads.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AdvancedLightingExtract {
    pub volumetric: Option<VolumetricFogSettings>,
    pub oit: Option<OitSettings>,
    pub fog_volumes: Vec<FogVolumeData>,
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
        self.volumetric.is_none()
            && self.oit.is_none()
            && self.fog_volumes.is_empty()
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
}

#[cfg(test)]
mod tests;
