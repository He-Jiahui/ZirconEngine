use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::core::framework::render::{
    AdvancedPbrMaterialFrameUsage, RenderFrameExtract, SubsurfaceProfileData,
};

/// Renderer-owned, view-local inputs that affect advanced-lighting graph compilation.
///
/// These values are resolved from visible materials without mutating the shared scene payload.
#[derive(Clone, Debug, Default)]
pub(crate) struct AdvancedLightingCompileInputs {
    material_features: AdvancedPbrMaterialFrameUsage,
    subsurface_profiles: Arc<[SubsurfaceProfileData]>,
    subsurface_material_profile_indices: Arc<[u32]>,
}

impl AdvancedLightingCompileInputs {
    pub(crate) fn new(
        material_features: AdvancedPbrMaterialFrameUsage,
        subsurface_profiles: Vec<SubsurfaceProfileData>,
        subsurface_material_profile_indices: Vec<u32>,
    ) -> Self {
        Self {
            material_features,
            subsurface_profiles: subsurface_profiles.into(),
            subsurface_material_profile_indices: subsurface_material_profile_indices.into(),
        }
    }

    pub(crate) fn from_extract(extract: &RenderFrameExtract) -> Self {
        let advanced_lighting = &extract.lighting.advanced_lighting;
        Self::new(
            advanced_lighting.material_features,
            advanced_lighting.subsurface_profiles.clone(),
            advanced_lighting
                .subsurface_material_profile_indices
                .clone(),
        )
    }

    pub(crate) const fn material_features(&self) -> AdvancedPbrMaterialFrameUsage {
        self.material_features
    }

    pub(crate) fn subsurface_profiles(&self) -> &[SubsurfaceProfileData] {
        &self.subsurface_profiles
    }

    pub(crate) fn subsurface_material_profile_indices(&self) -> &[u32] {
        &self.subsurface_material_profile_indices
    }

    pub(crate) fn transmission_scene_copy_step_count(&self, extract: &RenderFrameExtract) -> usize {
        if self.material_features.requires_scene_color_copy() {
            extract
                .lighting
                .advanced_lighting
                .screen_space_transmission
                .steps()
        } else {
            0
        }
    }

    pub(crate) fn transmission_draw_step_count(&self, extract: &RenderFrameExtract) -> usize {
        if !self.material_features.uses_transmission() {
            return 0;
        }
        self.transmission_scene_copy_step_count(extract).max(1)
    }
}

impl PartialEq for AdvancedLightingCompileInputs {
    fn eq(&self, other: &Self) -> bool {
        self.material_features == other.material_features
            && self.subsurface_profiles.len() == other.subsurface_profiles.len()
            && self
                .subsurface_profiles
                .iter()
                .zip(other.subsurface_profiles.iter())
                .all(|(left, right)| profile_bits_equal(left, right))
            && self.subsurface_material_profile_indices == other.subsurface_material_profile_indices
    }
}

impl Eq for AdvancedLightingCompileInputs {}

impl Hash for AdvancedLightingCompileInputs {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let material = self.material_features;
        material.clearcoat.hash(state);
        material.anisotropy.hash(state);
        material.dielectric_f0_override.hash(state);
        material.specular_transmission.hash(state);
        material.diffuse_transmission.hash(state);
        material.late_forward_opaque.hash(state);
        self.subsurface_profiles.len().hash(state);
        for profile in self.subsurface_profiles.iter() {
            profile.profile_id.hash(state);
            profile.scatter_radius_rgb.x.to_bits().hash(state);
            profile.scatter_radius_rgb.y.to_bits().hash(state);
            profile.scatter_radius_rgb.z.to_bits().hash(state);
            profile.falloff_rgb.x.to_bits().hash(state);
            profile.falloff_rgb.y.to_bits().hash(state);
            profile.falloff_rgb.z.to_bits().hash(state);
            profile.world_unit_scale.to_bits().hash(state);
        }
        self.subsurface_material_profile_indices.hash(state);
    }
}

fn profile_bits_equal(left: &SubsurfaceProfileData, right: &SubsurfaceProfileData) -> bool {
    left.profile_id == right.profile_id
        && left.scatter_radius_rgb.x.to_bits() == right.scatter_radius_rgb.x.to_bits()
        && left.scatter_radius_rgb.y.to_bits() == right.scatter_radius_rgb.y.to_bits()
        && left.scatter_radius_rgb.z.to_bits() == right.scatter_radius_rgb.z.to_bits()
        && left.falloff_rgb.x.to_bits() == right.falloff_rgb.x.to_bits()
        && left.falloff_rgb.y.to_bits() == right.falloff_rgb.y.to_bits()
        && left.falloff_rgb.z.to_bits() == right.falloff_rgb.z.to_bits()
        && left.world_unit_scale.to_bits() == right.world_unit_scale.to_bits()
}

#[cfg(test)]
mod tests {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::sync::Arc;

    use crate::core::framework::render::{AdvancedPbrMaterialFrameUsage, SubsurfaceProfileData};
    use crate::core::math::Vec3;

    use super::AdvancedLightingCompileInputs;

    #[test]
    fn runtime07_renderer_derived_lighting_inputs_clone_shares_variable_length_storage() {
        let inputs = inputs_with_profile_scale(1.0);
        let clone = inputs.clone();

        assert!(Arc::ptr_eq(
            &inputs.subsurface_profiles,
            &clone.subsurface_profiles
        ));
        assert!(Arc::ptr_eq(
            &inputs.subsurface_material_profile_indices,
            &clone.subsurface_material_profile_indices
        ));
    }

    #[test]
    fn runtime07_renderer_derived_lighting_inputs_hash_exact_profile_bits() {
        let baseline = inputs_with_profile_scale(1.0);
        let changed = inputs_with_profile_scale(f32::from_bits(1.0_f32.to_bits() + 1));

        assert_ne!(baseline, changed);
        assert_ne!(hash_of(&baseline), hash_of(&changed));
    }

    fn inputs_with_profile_scale(world_unit_scale: f32) -> AdvancedLightingCompileInputs {
        AdvancedLightingCompileInputs::new(
            AdvancedPbrMaterialFrameUsage {
                late_forward_opaque: true,
                ..Default::default()
            },
            vec![SubsurfaceProfileData::new(
                3,
                Vec3::new(0.8, 1.2, 1.8),
                Vec3::new(1.0, 0.45, 0.3),
                world_unit_scale,
            )],
            vec![3],
        )
    }

    fn hash_of(value: &AdvancedLightingCompileInputs) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }
}
