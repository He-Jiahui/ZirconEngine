use std::sync::Arc;

use super::super::OutputTargetTextureResource;

pub(in crate::graphics::scene::resources) struct PreparedOutputTargetTexture {
    pub(in crate::graphics::scene::resources) revision: u64,
    pub(in crate::graphics::scene::resources) resource: Arc<OutputTargetTextureResource>,
}

impl PreparedOutputTargetTexture {
    pub(in crate::graphics::scene::resources) const RETAINED_OUTPUT_TARGET_CACHE_OWNER_COUNT:
        usize = 1;

    pub(in crate::graphics::scene::resources) fn retained_output_target_cache_owner_count(
        &self,
    ) -> usize {
        let _retained_output_target_cache_owner = &self.resource;
        Self::RETAINED_OUTPUT_TARGET_CACHE_OWNER_COUNT
    }

    pub(in crate::graphics::scene::resources) fn resource(
        &self,
    ) -> &Arc<OutputTargetTextureResource> {
        debug_assert_eq!(
            self.retained_output_target_cache_owner_count(),
            Self::RETAINED_OUTPUT_TARGET_CACHE_OWNER_COUNT,
            "PreparedOutputTargetTexture must retain the output target resource while streamer exposes writeback and graph-import access",
        );
        &self.resource
    }
}
