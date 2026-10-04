//! 次表面配置从当前相机可见材质收敛到图编译输入，避免不可见材质改变通道结构。
use std::collections::BTreeMap;

use crate::asset::{Handle, MaterialAsset, ProjectAssetManager};
use crate::core::framework::render::{
    RenderFrameExtract, SubsurfaceProfileData, ZR_SSS_MAX_PROFILES,
};

/// Completes the production frame sideband from materials visible to this
/// submission. Explicit scene-owned profiles win over embedded material data.
pub(super) fn resolve_subsurface_material_profiles(
    asset_manager: &ProjectAssetManager,
    extract: &RenderFrameExtract,
) -> (Vec<SubsurfaceProfileData>, Vec<u32>) {
    let mut profiles_by_id = extract
        .lighting
        .advanced_lighting
        .subsurface_profiles
        .iter()
        .copied()
        .map(|profile| (profile.profile_id, profile))
        .collect::<BTreeMap<_, _>>();
    let materials = asset_manager.assets::<MaterialAsset>();
    let mut used_profile_mask = 0_u32;

    for mesh in &extract.geometry.meshes {
        let Some(material) = materials.get(Handle::from_resource_handle(mesh.material)) else {
            continue;
        };
        if !material.is_subsurface_material() {
            continue;
        }
        let profile_id = material.subsurface_profile_index();
        debug_assert!(profile_id < ZR_SSS_MAX_PROFILES as u32);
        used_profile_mask |= 1_u32 << profile_id;
        if let Some(profile) = material.authored_subsurface_profile() {
            profiles_by_id.entry(profile_id).or_insert(profile);
        }
    }

    (
        profiles_by_id.into_values().collect(),
        subsurface_profile_indices_from_mask(used_profile_mask),
    )
}

fn subsurface_profile_indices_from_mask(active_mask: u32) -> Vec<u32> {
    (0..ZR_SSS_MAX_PROFILES as u32)
        .filter(|profile_id| active_mask & (1_u32 << profile_id) != 0)
        .collect()
}

#[cfg(test)]
#[path = "tests/subsurface_profile_extract_optimization_tests.rs"]
mod optimization_tests;
