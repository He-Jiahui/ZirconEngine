use zircon_runtime::core::framework::render::RenderMeshBounds;
use zircon_runtime::core::math::Vec3;

use super::declarations::{
    HybridGiGlobalSdfPageBuildRequest, HybridGiGlobalSdfPageKey, HybridGiGlobalSdfSceneState,
};

impl HybridGiGlobalSdfSceneState {
    pub(in crate::hybrid_gi) fn dirty_page_build_requests(
        &self,
    ) -> Vec<HybridGiGlobalSdfPageBuildRequest> {
        self.dirty_pages
            .iter()
            .filter_map(|key| {
                self.resident_pages
                    .get(key)
                    .map(|page| HybridGiGlobalSdfPageBuildRequest {
                        key: *key,
                        requested_generation: page.generation,
                        atlas_slot: page.atlas_slot,
                    })
            })
            .collect()
    }

    pub(in crate::hybrid_gi) fn sampleable_pages(
        &self,
    ) -> impl Iterator<Item = HybridGiGlobalSdfPageBuildRequest> + '_ {
        self.resident_pages
            .iter()
            .filter(|(key, page)| {
                page.initialized
                    && !self.dirty_pages.contains(key)
                    && !self
                        .influence_index
                        .clipmap_uses_voxel_fallback(key.clipmap_id)
            })
            .map(|(key, page)| HybridGiGlobalSdfPageBuildRequest {
                key: *key,
                requested_generation: page.generation,
                atlas_slot: page.atlas_slot,
            })
    }

    pub(in crate::hybrid_gi) fn commit_pages(
        &mut self,
        completions: &[HybridGiGlobalSdfPageBuildRequest],
    ) {
        for completion in completions {
            let Some(page) = self.resident_pages.get_mut(&completion.key) else {
                continue;
            };
            if page.generation != completion.requested_generation
                || !self.dirty_pages.contains(&completion.key)
            {
                continue;
            }
            page.initialized = true;
            self.dirty_pages.remove(&completion.key);
        }
    }

    pub(in crate::hybrid_gi) fn resolve_pages_to_fallback(
        &mut self,
        requests: &[HybridGiGlobalSdfPageBuildRequest],
    ) {
        for request in requests {
            let Some(page) = self.resident_pages.get_mut(&request.key) else {
                continue;
            };
            if page.generation != request.requested_generation
                || !self.dirty_pages.contains(&request.key)
            {
                continue;
            }
            page.initialized = false;
            self.dirty_pages.remove(&request.key);
        }
    }

    pub(in crate::hybrid_gi) fn clipmap_bounds(&self) -> &[super::HybridGiGlobalSdfClipmapBounds] {
        &self.clipmap_bounds
    }

    pub(in crate::hybrid_gi) fn resident_page_count(&self) -> usize {
        self.resident_pages.len()
    }

    pub(in crate::hybrid_gi) fn resident_page_keys(&self) -> Vec<HybridGiGlobalSdfPageKey> {
        self.resident_pages.keys().copied().collect()
    }

    pub(in crate::hybrid_gi) fn dirty_page_keys(&self) -> Vec<HybridGiGlobalSdfPageKey> {
        self.dirty_pages.iter().copied().collect()
    }

    pub(in crate::hybrid_gi) fn is_page_sampleable(&self, key: HybridGiGlobalSdfPageKey) -> bool {
        self.resident_pages.get(&key).is_some_and(|page| {
            page.initialized
                && !self.dirty_pages.contains(&key)
                && !self
                    .influence_index
                    .clipmap_uses_voxel_fallback(key.clipmap_id)
        })
    }

    pub(in crate::hybrid_gi) fn page_candidate_keys(
        &self,
        key: HybridGiGlobalSdfPageKey,
    ) -> Option<&[u64]> {
        self.influence_index.page_candidate_keys(key)
    }

    pub(in crate::hybrid_gi) fn page_has_candidate_overflow(
        &self,
        key: HybridGiGlobalSdfPageKey,
    ) -> bool {
        self.influence_index.page_has_candidate_overflow(key)
    }

    pub(in crate::hybrid_gi) fn clipmap_uses_voxel_fallback(&self, clipmap_id: u32) -> bool {
        self.influence_index.clipmap_uses_voxel_fallback(clipmap_id)
    }

    pub(in crate::hybrid_gi) fn page_bounds(
        &self,
        key: HybridGiGlobalSdfPageKey,
    ) -> Option<RenderMeshBounds> {
        let clipmap = self
            .clipmap_bounds
            .iter()
            .copied()
            .find(|clipmap| clipmap.clipmap_id == key.clipmap_id)?;
        let page_world_size = clipmap.page_world_size();
        let min = Vec3::new(
            key.page_coordinate[0] as f32,
            key.page_coordinate[1] as f32,
            key.page_coordinate[2] as f32,
        ) * page_world_size;
        let max = min + Vec3::splat(page_world_size);
        Some(RenderMeshBounds::from_min_max(
            min.to_array(),
            max.to_array(),
        ))
    }

    pub(in crate::hybrid_gi) fn page_influence_bounds(
        &self,
        key: HybridGiGlobalSdfPageKey,
    ) -> Option<RenderMeshBounds> {
        let bounds = self.page_bounds(key)?;
        let page_extent = bounds.max[0] - bounds.min[0];
        if !page_extent.is_finite() || page_extent <= 0.0 {
            return None;
        }
        Some(RenderMeshBounds::from_min_max(
            bounds.min.map(|value| value - page_extent),
            bounds.max.map(|value| value + page_extent),
        ))
    }
}
