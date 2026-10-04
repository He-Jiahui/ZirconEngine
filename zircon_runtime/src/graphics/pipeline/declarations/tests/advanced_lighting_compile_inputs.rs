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
