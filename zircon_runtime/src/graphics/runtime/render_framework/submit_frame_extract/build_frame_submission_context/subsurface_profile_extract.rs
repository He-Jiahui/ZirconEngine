use std::collections::{BTreeMap, BTreeSet};

use crate::asset::{Handle, MaterialAsset, ProjectAssetManager};
use crate::core::framework::render::{RenderFrameExtract, SubsurfaceProfileData};

/// Completes the production frame sideband from materials visible to this
/// submission. Explicit scene-owned profiles win over embedded material data.
pub(super) fn resolve_subsurface_material_profiles(
    asset_manager: &ProjectAssetManager,
    extract: &mut RenderFrameExtract,
) {
    let mut profiles_by_id = extract
        .lighting
        .advanced_lighting
        .subsurface_profiles
        .iter()
        .copied()
        .map(|profile| (profile.profile_id, profile))
        .collect::<BTreeMap<_, _>>();
    let materials = asset_manager.assets::<MaterialAsset>();
    let mut used_profile_ids = BTreeSet::new();

    for mesh in &extract.geometry.meshes {
        let Some(material) = materials.get(Handle::from_resource_handle(mesh.material)) else {
            continue;
        };
        if !material.is_subsurface_material() {
            continue;
        }
        let profile_id = material.subsurface_profile_index();
        used_profile_ids.insert(profile_id);
        if let Some(profile) = material.authored_subsurface_profile() {
            profiles_by_id.entry(profile_id).or_insert(profile);
        }
    }

    extract.lighting.advanced_lighting.subsurface_profiles = profiles_by_id
        .into_values()
        .collect::<Vec<SubsurfaceProfileData>>();
    extract
        .lighting
        .advanced_lighting
        .subsurface_material_profile_indices = used_profile_ids.into_iter().collect();
}
