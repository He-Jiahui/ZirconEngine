use crate::core::framework::render::{RenderMaterialLightingModel, SubsurfaceProfileData};
use crate::core::math::Vec3;

use super::{MaterialAsset, material_control, override_f32, override_vec3};

impl MaterialAsset {
    pub fn subsurface_profile_index(&self) -> u32 {
        material_control::subsurface_profile_index(&self.property_values).unwrap_or_default()
    }

    pub fn is_subsurface_material(&self) -> bool {
        matches!(
            self.lighting_model(),
            RenderMaterialLightingModel::Custom { ref name }
                if name.eq_ignore_ascii_case("subsurface")
        )
    }

    /// Returns an embedded profile authored on a Subsurface material. A scene
    /// producer may instead provide the same profile id through its explicit
    /// advanced-lighting extract.
    pub fn authored_subsurface_profile(&self) -> Option<SubsurfaceProfileData> {
        if !self.is_subsurface_material() {
            return None;
        }
        let radius = override_vec3(&self.property_values, "subsurface_scatter_radius")?;
        let falloff =
            override_vec3(&self.property_values, "subsurface_falloff").unwrap_or([1.0, 1.0, 1.0]);
        let world_unit_scale =
            override_f32(&self.property_values, "subsurface_world_unit_scale").unwrap_or(0.001);
        Some(SubsurfaceProfileData::new(
            self.subsurface_profile_index(),
            Vec3::new(radius[0], radius[1], radius[2]),
            Vec3::new(falloff[0], falloff[1], falloff[2]),
            world_unit_scale,
        ))
    }
}
