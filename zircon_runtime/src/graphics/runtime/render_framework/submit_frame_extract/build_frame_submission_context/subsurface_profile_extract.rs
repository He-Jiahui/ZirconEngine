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
mod optimization_tests {
    use std::collections::BTreeSet;
    use std::hint::black_box;
    use std::mem::size_of;
    use std::time::Instant;

    use super::*;
    use crate::core::framework::render::RenderWorldSnapshotHandle;
    use crate::scene::world::World;

    #[test]
    fn runtime07_renderer_derived_lighting_subsurface_resolver_does_not_mutate_extract() {
        let manager = ProjectAssetManager::default();
        let extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(1),
            World::new().to_render_snapshot(),
        );

        let (profiles, indices) = resolve_subsurface_material_profiles(&manager, &extract);

        assert!(profiles.is_empty());
        assert!(indices.is_empty());
        assert!(extract
            .lighting
            .advanced_lighting
            .subsurface_profiles
            .is_empty());
        assert!(extract
            .lighting
            .advanced_lighting
            .subsurface_material_profile_indices
            .is_empty());
    }

    #[test]
    fn optimization_batch_20260826i_runtime99a_profile_mask_preserves_sorted_unique_slots() {
        let mask = (1_u32 << 15) | (1_u32 << 2) | (1_u32 << 7);

        assert_eq!(subsurface_profile_indices_from_mask(mask), vec![2, 7, 15]);
    }

    #[test]
    fn optimization_batch_20260826i_runtime99a_profile_usage_uses_fixed_capacity_mask() {
        let source = include_str!("subsurface_profile_extract.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("subsurface profile production source");
        let resolver = production
            .split("pub(super) fn resolve_subsurface_material_profiles")
            .nth(1)
            .expect("subsurface profile resolver")
            .split("fn subsurface_profile_indices_from_mask")
            .next()
            .expect("bounded subsurface profile resolver");

        assert!(!production.contains("BTreeSet"));
        assert!(resolver.contains("let mut used_profile_mask = 0_u32"));
        assert!(resolver.contains("used_profile_mask |= 1_u32 << profile_id"));
        assert!(production.contains("0..ZR_SSS_MAX_PROFILES as u32"));
    }

    #[test]
    #[ignore = "release performance evidence; run through the validation coordinator"]
    fn optimization_batch_20260826i_runtime99a_profile_mask_dedup_performance_evidence() {
        fn legacy_normalize(profile_ids: &[u32]) -> Vec<u32> {
            profile_ids
                .iter()
                .copied()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect()
        }

        fn mask_normalize(profile_ids: &[u32]) -> Vec<u32> {
            let mut mask = 0_u32;
            for profile_id in profile_ids {
                mask |= 1_u32 << *profile_id;
            }
            subsurface_profile_indices_from_mask(mask)
        }

        let profile_ids = (0..32_768_u32)
            .map(|index| index.wrapping_mul(0x9E37_79B9) & 0xF)
            .collect::<Vec<_>>();
        let mut legacy_samples = Vec::with_capacity(17);
        let mut mask_samples = Vec::with_capacity(17);
        for _ in 0..17 {
            let started = Instant::now();
            black_box(legacy_normalize(black_box(&profile_ids)));
            legacy_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(mask_normalize(black_box(&profile_ids)));
            mask_samples.push(started.elapsed().as_nanos());
        }

        legacy_samples.sort_unstable();
        mask_samples.sort_unstable();
        let legacy_p95 = legacy_samples[16];
        let mask_p95 = mask_samples[16];
        println!(
            "RUNTIME99A_SUBSURFACE_PROFILE_MASK_DEDUP_BENCH_V1 mesh_profile_refs={} unique_slots={} legacy_p95_ns={} mask_p95_ns={} legacy_tree_insertions={} mask_bit_sets={} legacy_transient_tree_entries={} mask_storage_bytes={} target_ratio_bp=6000",
            profile_ids.len(),
            ZR_SSS_MAX_PROFILES,
            legacy_p95,
            mask_p95,
            profile_ids.len(),
            profile_ids.len(),
            ZR_SSS_MAX_PROFILES,
            size_of::<u32>(),
        );
        assert!(
            mask_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(6_000),
            "profile mask P95 {mask_p95} ns exceeded 60% of legacy {legacy_p95} ns"
        );
    }
}
