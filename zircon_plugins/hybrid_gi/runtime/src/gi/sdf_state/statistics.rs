use super::declarations::HybridGiGlobalSdfSceneState;

impl HybridGiGlobalSdfSceneState {
    pub(in crate::hybrid_gi) fn dirty_page_count(&self) -> usize {
        self.dirty_pages.len()
    }

    pub(in crate::hybrid_gi) fn sampleable_page_count(&self) -> usize {
        self.sampleable_pages().count()
    }

    pub(in crate::hybrid_gi) fn candidate_contributor_count(&self) -> usize {
        self.influence_index.candidate_contributor_count()
    }

    pub(in crate::hybrid_gi) fn clipmap_fallback_count(&self) -> usize {
        self.influence_index.voxel_fallback_clipmap_count()
    }

    pub(in crate::hybrid_gi) fn candidate_bucket_capacity_bytes(&self) -> u64 {
        self.influence_index.candidate_bucket_capacity_bytes()
    }
}
